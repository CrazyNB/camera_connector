use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    ProjectEvaluationSettings, PromptPack, PromptPackContent, Result, SceneProfile, SqliteStore,
};

use super::{stable_prompt_hash, MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION};

#[derive(Debug, Clone)]
pub(super) struct PromptSnapshot {
    pub(super) prompt_pack_id: String,
    pub(super) prompt_pack_version: String,
    pub(super) prompt_hash: String,
    pub(super) prompt_content: PromptPackContent,
}

pub(super) fn prompt_snapshot_for_settings(
    store: &SqliteStore,
    settings: &ProjectEvaluationSettings,
) -> Result<Option<PromptSnapshot>> {
    let Some(prompt_pack_id) = settings.prompt_pack_id.as_deref() else {
        return Ok(None);
    };
    let pack = match builtin_prompt_packs()
        .into_iter()
        .find(|pack| pack.prompt_pack_id == prompt_pack_id)
    {
        Some(pack) => pack,
        None => load_user_prompt_packs(&store.state_dir())?
            .into_iter()
            .find(|pack| pack.prompt_pack_id == prompt_pack_id)
            .ok_or_else(|| crate::ImporterError::internal("prompt pack not found"))?,
    };
    Ok(Some(PromptSnapshot {
        prompt_pack_id: pack.prompt_pack_id,
        prompt_pack_version: pack.version,
        prompt_hash: pack.prompt_hash,
        prompt_content: prompt_pack_content_from_json(&pack.prompt_text)?,
    }))
}

pub(super) fn builtin_prompt_packs() -> Vec<PromptPack> {
    let definitions = [
        (
            "general-default",
            "通用评价",
            "Camera Connector",
            vec!["通用".to_string(), "均衡".to_string(), "基础".to_string()],
            SceneProfile::General,
            r#"## 评分维度
- 主体价值: 优先判断画面有没有清楚的观看理由，主体是否能在一秒内被识别，主体和环境之间是否有关系，而不是只看照片是否清晰。
- 光线与色彩: 看光线方向、层次、色温和色彩关系是否服务主题。允许低调、高反差、阴天、逆光，但不能把偶然的曝光错误误判成风格。
- 构图与视觉秩序: 关注边缘、前景、背景、线条、留白、重心、遮挡和干扰物。好照片应让视线自然停留在主体和关键动作上。
- 技术底线: 焦点、抖动、过曝、死黑、噪点、偏色、压缩痕迹都要检查。技术问题如果削弱主体信息，应明显降分；如果是合理表达，可保留。
- 情绪与记忆点: 评价照片是否有情绪、气氛、关系、瞬间或形式上的独特性，避免只奖励“安全漂亮”的图。

## 淘汰
- 主体不存在、主体被关键遮挡、焦点落错、重要脸部表情不可读、画面明显歪斜且无表达理由，优先淘汰。
- 多张相似照片中，只因亮度稍亮或颜色更艳而胜出的图要谨慎，避免选出过度处理、廉价讨喜但信息弱的照片。
- 若质量风险来自局部小区域，应结合主体重要性判断；若风险出现在主体脸部、文字、动作峰值或商业交付重点，降分更重。

## 连拍
- 先找决定性瞬间: 表情、眼神、手势、动作峰值、人物关系、主体朝向和背景分离度。
- 同一组内不要只按锐度排序。若一张略软但情绪、动作、构图明显更好，可以高于完全清晰但无内容的照片。
- 连拍优选要给出保留理由和淘汰理由，说明最终选择解决了什么观看问题。"#,
        ),
        (
            "portrait-conservative",
            "人像稳健",
            "Camera Connector",
            vec!["人像".to_string(), "稳健".to_string(), "交付".to_string()],
            SceneProfile::Portrait,
            r#"## 评分维度
- 人物状态: 眼神、表情、嘴型、姿态和手部是否自然，人物是否有尊严感和可信度。优先选择让被摄者看起来舒服、真实、可信的照片。
- 面部质量: 眼睛焦点、面部曝光、肤色、闭眼、眨眼、表情僵硬、面部遮挡和局部死黑死白是核心检查点。脸部局部风险应直接影响可用性。
- 关系与语境: 判断人物和环境、道具、光线之间是否建立身份或情绪线索。商业人像可以更干净，纪实人像可以保留环境复杂度，但都要服务人物。
- 光线与修饰: 保留真实肤色和面部体积，避免过度磨皮、过度锐化、脏色阴影和不自然白平衡。发丝、眼镜反光、服装褶皱要按交付场景权衡。
- 稳定交付: 这套规则偏保守，适合客户交付、活动人像、证件化肖像和不希望冒险的项目。艺术实验可以保留，但必须明显提升表达。

## 淘汰
- 闭眼、半眨眼、眼神失焦、嘴型尴尬、脸部关键区域糊掉、面部过曝或死黑，原则上淘汰。
- 表情相近时，淘汰脸部变形、姿态紧张、背景穿头、手势怪异、眼镜严重反光或肤色明显偏脏的版本。
- 不要让“背景更漂亮”压过人物状态。人像项目中，人物可信度优先于风景和装饰。

## 连拍
- 连拍内优先比较眼神和表情微差，其次比较脸部清晰度、手部位置、肩颈线条、背景遮挡。
- 若多张都可用，选择最稳妥的一张作为主选，保留一张情绪更强但风险略高的备选说明。
- 给出不选其他帧的具体原因，例如眨眼、嘴型、焦点、脸部阴影、背景干扰。"#,
        ),
        (
            "landscape-technical",
            "风光技术",
            "Camera Connector",
            vec!["风光".to_string(), "技术".to_string(), "清晰".to_string()],
            SceneProfile::Landscape,
            r#"## 评分维度
- 光线质量: 风光照片首先看光线是否塑造空间。日出日落、云层、雾气、逆光、侧光、阴天都可以成立，但需要有层次和方向，而不是平、灰、脏。
- 空间层次: 前景、中景、远景是否建立深度，地平线、山脊、河流、道路、树线是否形成视线引导。空镜也需要视觉节奏和停留点。
- 技术完整度: 检查地平线、边角锐度、主体锐度、动态范围、云层高光、暗部细节、色带、噪点和镜头污点。技术问题在大幅输出时权重更高。
- 色彩与调性: 避免过饱和、过度 HDR、青橙套色、天空和地面割裂。自然风光允许艺术化，但色彩要有一致的空气感。
- 场景真实性: 如果画面依赖极端后期、假天空、明显合成或违背现场光线关系，应降低可信度。

## 淘汰
- 地平线无理由倾斜、天空大面积死白、暗部完全堵死、主体或关键纹理糊掉、画面没有明确视觉路径，优先淘汰。
- 多张风光相似时，淘汰光线平淡、边缘干扰明显、色彩过重、前景杂乱或没有空间层次的版本。
- 不要只奖励“最亮、最艳、最锐”的图。若处理感破坏自然空气和深度，应低于克制但有气氛的版本。

## 连拍
- 连拍或包围曝光组内，优先选择光线峰值、云影位置、人物或动物进入画面的瞬间、风吹草木的形态和水面反射完整度。
- 若技术最佳帧和情绪最佳帧不同，说明取舍；项目优选时可推荐技术稳定的主图和气氛更强的副图。
- 连拍筛选要保留能代表同一场景变化的少数帧，避免重复保留只差曝光半档的照片。"#,
        ),
        (
            "documentary-integrity",
            "纪实真实",
            "Camera Connector",
            vec!["纪实".to_string(), "新闻".to_string(), "叙事".to_string()],
            SceneProfile::General,
            r#"## 评分维度
- 事实与伦理: 纪实照片先看可信度。画面应尊重被摄者和事件，不鼓励摆拍伪装成现场，不鼓励误导性裁切、过度后期、删除关键语境或制造不存在的关系。
- 信息密度: 好的纪实照片需要让观众理解人物、地点、动作、环境和冲突。主体不一定漂亮，但必须提供真实信息和情绪线索。
- 决定性瞬间: 关注动作临界点、表情变化、人物关系、手势、眼神、事件发展方向。瞬间越不可重复，权重越高。
- 叙事结构: 单张要能独立成立，系列或项目优选要考虑开场、推进、转折、细节、收束。不要全选同一种距离和情绪。
- 形式服务内容: 构图、光线、色彩、颗粒、模糊、遮挡都可以不完美，但必须增强现场感和叙事，不应只是失误。

## 淘汰
- 画面漂亮但信息空洞、人物被消费化、事件语境不清、关键动作缺失、后期明显误导，应淘汰或降级。
- 如果一张照片可能误导事实，例如裁掉关键参与者、改变事件方向、把普通场景渲染成灾难，应避免推荐。
- 技术瑕疵不是唯一淘汰理由；但主体不可读、事实线索被破坏、表情动作错失，仍应淘汰。

## 连拍
- 连拍中优先找事件关系最完整的一帧: 谁在做什么、为什么重要、下一秒会发生什么。
- 表情和动作峰值高于单纯清晰度。若某帧略糊但抓住不可重复关系，可以推荐，同时说明风险。
- 项目级选择要控制重复，保留不同距离、角色和信息功能的照片，让组图能讲完整故事。
- 输出结论时要说明这张图承担的叙事功能，例如建立地点、呈现人物、解释冲突、提供细节或完成收束。"#,
        ),
        (
            "portrait-editorial",
            "人像编辑",
            "Camera Connector",
            vec!["人像".to_string(), "编辑".to_string(), "情绪".to_string()],
            SceneProfile::Portrait,
            r#"## 评分维度
- 身份表达: 这套规则更偏杂志、封面、专题和作者型人像。优先判断照片是否揭示被摄者的性格、身份、关系或心理状态，而不是只看好看。
- 眼神与姿态: 眼神方向、面部微表情、肩颈、手、身体重心和服装轮廓共同构成人物叙事。微妙的不完美可以保留，只要它让人物更有张力。
- 光线风格: 接受低调、硬光、彩色光、环境光和戏剧化阴影，但面部关键区域必须可读。脸部风险要在评价中被明确说明。
- 环境与造型: 背景、道具、衣服、发型和空间线条应帮助人物成立。复杂背景可以成立，但不能和人物争夺叙事中心。
- 编辑价值: 评价照片是否适合作为封面、专题开篇、人物档案或社交传播主图。优先选择能被记住的图，而非最保险的图。

## 淘汰
- 情绪平、姿态空、眼神漂、人物和环境无关系、面部不可读或造型细节严重破坏人物质感，应淘汰。
- 不要因为肤色更亮、背景更干净就自动胜出；如果另一张更有心理张力，应该给更高分。
- 明显冒犯、丑化、误读被摄者身份的照片，即使视觉强，也不应推荐为主选。

## 连拍
- 连拍中比较微表情、眼神角度、下颌线、手势、服装边缘、背景穿插和光斑位置。
- 可保留一张“安全交付”和一张“编辑张力”不同用途的候选，但必须明确主推逻辑。
- 组图优选时避免全是同一表情，选择能构成情绪变化的帧。
- 输出结论时区分“客户稳妥可用”和“编辑视觉更强”，不要把两种用途混成一个模糊分数。"#,
        ),
        (
            "portrait-lifestyle",
            "写真创作",
            "Camera Connector",
            vec![
                "写真".to_string(),
                "情绪".to_string(),
                "风格".to_string(),
                "生活方式".to_string(),
            ],
            SceneProfile::Portrait,
            r#"## 评分维度
- 风格完成度: 这套规则面向写真、约拍、旅拍、情绪人像和生活方式创作。先判断照片是否有明确风格意图，而不是只看脸是否好看。可参考亲密日记、日系空气感、青春动态、光影剪影、几何色块、广角近距离冲击等成熟作品方向，但不要要求照片模仿某个摄影师。
- 故事感: 判断人物和场景之间是否有关系，例如等待、奔跑、回头、触碰、沉默、旅途、房间痕迹、街道偶遇、季节和时间。好写真应让观众相信照片前后还有故事。
- 情绪与距离: 关注眼神、身体松紧、人与镜头的距离、表情克制程度、孤独感、亲密感、松弛感或不安感。情绪可以轻、淡、冷、甜、躁动，但不能空。
- 日系与清新: 若照片走自然光、低反差、浅色、留白、生活化细节路线，应优先评价空气感、肤色自然度、光线柔和度和画面呼吸感。过曝可以成立，但不能丢掉脸部和关键动作信息。
- 运动与动态: 若照片强调奔跑、跳跃、风吹、旋转、骑行、海边或街头动作，应判断身体线条、动作峰值、速度方向和快门取舍。轻微动态模糊可以增强生命力，但不能让主体状态不可读。
- 广角与冲击: 若使用广角、近距离、低机位或强透视，应判断夸张是否服务人物张力和空间关系。边缘变形、肢体拉伸、脸部变形如果破坏人物质感，应降分；如果增强现场压迫感和能量，可以保留。
- 剪影与逆光: 若走剪影、逆光、暗部轮廓路线，应看姿态是否一眼可读、轮廓是否干净、背景是否有层次、曝光取舍是否有意图。脸部不可见不是问题，人物状态不可辨才是问题。

## 淘汰
- 只有滤镜、只有漂亮背景、只有摆姿势但没有情绪或故事感的照片，应降级或淘汰。
- 写真不等于无限宽容。焦点落错、脸部状态尴尬、肢体变形难看、背景穿头、服装细节破坏、肤色脏、画面廉价套色，都要明确扣分。
- 日系清新照片若只是低对比发灰、过曝丢信息、人物无状态，不应高分；广角冲击若只是变形和贴脸，也不应高分。
- 剪影照片若轮廓粘连、动作不可读、背景杂乱、主体和环境没有关系，应淘汰。
- 对明显消费化、冒犯、丑化被摄者或让人物失去尊严的照片，即使风格强，也不推荐。

## 连拍
- 连拍中优先比较情绪微差、眼神方向、身体线条、动作峰值、发丝和衣摆形态、手部位置、背景遮挡和光线落点。
- 故事感写真要选“最像一个瞬间”的帧，而不是最像摆拍定格的帧。若一张轻微不完美但有真实状态，可以高于更端正但空洞的版本。
- 日系或情绪路线中，不要只选最亮、最白、最干净的一张；保留空气、节奏、留白和人物松弛感。
- 运动或广角路线中，优先选择动作峰值、空间张力和脸部/身体形态同时成立的一帧。
- 剪影或逆光路线中，优先选择轮廓最清楚、姿态最有识别度、背景层次最好的一帧。
- 输出结论时必须说明这张图属于哪种写真风格路径，以及它胜出的核心原因: 故事感、情绪、日系空气感、运动能量、广角冲击或剪影轮廓。"#,
        ),
        (
            "landscape-fine-art",
            "风光艺术",
            "Camera Connector",
            vec!["风光".to_string(), "艺术".to_string(), "氛围".to_string()],
            SceneProfile::Landscape,
            r#"## 评分维度
- 气氛优先: 这套规则重视画面的诗意、沉浸感和可停留性。雾、雨、雪、逆光、低对比、极简空间都可以高分，只要它们构成完整的视觉情绪。
- 构成关系: 关注形状、线条、明暗块面、负空间、节奏、比例和视觉重量。主体可以很小，但画面必须有秩序。
- 光影层次: 好的艺术风光不只是清晰，还要有空气透视、明暗过渡、细节取舍和观看路径。过度锐化或 HDR 会降低高级感。
- 色彩克制: 色彩可以浓烈，但要有整体调性。避免艳俗饱和、天空和地面色温冲突、局部色块抢戏。
- 作者性: 判断这张图是否有个人选择，而不是旅游打卡模板。独特天气、视角、时间、前景遮挡和抽象化处理都可以加分。

## 淘汰
- 空洞的明信片视角、过度后期、没有层次的灰片、只有景点没有表达、边缘杂乱且没有意图，应淘汰。
- 若技术完美但没有气氛和结构，不应高于有表达但局部技术略有瑕疵的照片。
- 明显倾斜、脏点、色带、天空断层和锐化光晕会破坏艺术输出，应大幅降分。

## 连拍
- 连拍中比较光线落点、云雾形态、水面反射、人物或鸟进入画面的比例，以及画面呼吸感。
- 风光艺术优选可以保留相邻两张不同情绪的照片，但同一构图的重复帧应严格压缩。
- 给出“为什么这一帧更有气氛”的理由，而不是只说更清晰或更亮。
- 项目级推荐要让颜色、天气和空间节奏形成连续观看体验，避免把彼此不兼容的调性硬放在一起。
- 如果某张图更安静、更留白但能建立全组气质，应允许它高于单张冲击力更强却破坏节奏的照片。"#,
        ),
        (
            "wildlife-ethics",
            "野生自然",
            "Camera Connector",
            vec!["野生".to_string(), "自然".to_string(), "伦理".to_string()],
            SceneProfile::Landscape,
            r#"## 评分维度
- 行为瞬间: 野生自然优先看真实行为、互动、觅食、迁徙、守护、警觉、运动和生境关系。动物静态肖像也可以成立，但必须有姿态、眼神或环境信息。
- 伦理与距离: 不鼓励诱拍、惊扰、捕捉受困动物、破坏栖息地或让动物呈现异常压力。若画面显得像圈养、摆拍或过度接近，应谨慎降分。
- 生境叙事: 好照片应让观众看到动物与环境的关系，包括季节、天气、植被、地貌和人类活动痕迹。
- 技术与主体: 眼部焦点、羽毛或毛发细节、动作冻结或合理动态模糊、背景分离、噪点和远距离裁切质量都很重要。
- 稀缺性与原创性: 不仅看物种稀有，也看视角、行为和故事是否少见。常见物种拍出新关系也可以高分。

## 淘汰
- 动物眼睛不可读、主体过小且无环境叙事、行为缺失、严重裁切糊化、背景杂乱抢戏，优先淘汰。
- 任何暗示诱捕、骚扰、危险接近或不自然摆布的画面，不应推荐为最佳。
- 不要用“物种稀有”掩盖摄影失败。稀有物种但画面弱，应低于普通物种的强瞬间。

## 连拍
- 连拍中选择动作最高点、眼神最清楚、身体姿态最完整、背景最干净的一帧。
- 如果连续动作构成行为故事，项目级可保留起势、峰值和结果三类帧；单组推荐只选最有信息的一张。
- 说明淘汰帧是否因为眼神、翅膀姿态、遮挡、背景、焦点或伦理风险。
- 若照片涉及人类投喂、围观、围栏、表演或明显人工控制环境，要在推荐理由里降低其野生可信度。
- 输出时同时说明行为价值和伦理风险，不要只用“可爱”“稀有”“漂亮”作为推荐依据。"#,
        ),
        (
            "action-sports-moment",
            "运动瞬间",
            "Camera Connector",
            vec!["运动".to_string(), "动作".to_string(), "速度".to_string()],
            SceneProfile::Action,
            r#"## 评分维度
- 峰值动作: 运动和动作照片优先看动作是否到达临界点，例如起跳最高点、冲线、碰撞、挥拍击球、转身、摔倒前后或情绪爆发。
- 身体线条: 肢体形态、脸部朝向、手脚位置、器材位置和主体完整度决定画面力量。动作被截断或姿态尴尬会明显降分。
- 速度表达: 清晰冻结和动态模糊都可以成立。关键是观众能感到速度、力量、方向和风险，而不是单纯糊。
- 背景分离: 体育场、人群、广告牌、裁判、其他运动员都可能干扰主体。好照片应让动作从背景中跳出来。
- 情绪与结果: 表情、胜负关系、团队互动和观众反应可以让动作照片从记录变成故事。

## 淘汰
- 错过动作峰值、球或器材离开关键关系、主体被遮挡、脸部不可读、肢体切割尴尬、背景严重干扰，优先淘汰。
- 只因更清晰而选择动作无力的帧，是错误取舍。动作能量和事件信息要高于安全锐度。
- 若动态模糊没有方向感，只是抖动或失焦，应视为质量问题而不是风格。

## 连拍
- 连拍中按事件曲线选择: 起势、接触、峰值、结果。单张推荐通常选峰值，组图推荐可以保留完整动作链。
- 对比帧时重点看球、眼神、手脚、身体张力、背景分离和表情，不要只按时间顺序选中间帧。
- 给出明确说明: 这一帧为什么是动作峰值，其他帧输在哪里。
- 项目级选择要平衡胜负情绪、动作种类、人物身份和场地信息，避免全是相同动作的安全帧。
- 若动作主体很小但环境能说明赛事规模或危险性，可以作为项目辅助帧，但不要替代真正的峰值动作主图。"#,
        ),
        (
            "architecture-design",
            "建筑空间",
            "Camera Connector",
            vec!["建筑".to_string(), "空间".to_string(), "秩序".to_string()],
            SceneProfile::Custom,
            r#"## 评分维度
- 空间秩序: 建筑照片看线条、比例、透视、尺度、体块、材料和光线如何组织空间。画面应让人理解建筑关系，而不只是记录外观。
- 透视控制: 垂直线、水平线、边缘裁切、广角变形和消失点需要被认真判断。透视可以夸张，但必须服务空间表达。
- 光影与材质: 光线应揭示材料质感、结构深度和空间层次。过平的光会让建筑失去体积，过重后期会让材料不可信。
- 人与尺度: 人物、家具、植物、道路、窗户和阴影可以提供尺度与使用痕迹。没有人也可以成立，但需要更强的形式秩序。
- 项目叙事: 建筑系列要覆盖外观、入口、空间转折、细节、使用场景和环境关系，避免全是同一角度的立面。

## 淘汰
- 无理由歪斜、垂直线明显失控、空间关系混乱、主体建筑被遮挡、边缘裁切粗糙、材料颜色失真，应淘汰或降分。
- 只因天空漂亮或滤镜强烈而忽略建筑本体，是错误选择。建筑主体和空间逻辑必须优先。
- 室内外高反差时，窗外死白或室内死黑若破坏空间信息，应降分。

## 连拍
- 连拍中选择人流位置、光影落点、门窗开启、反射和阴影最能说明空间的一帧。
- 同一机位的多张照片只保留结构最清楚、干扰最少、尺度最好的一张。
- 项目级推荐要形成“远景到细节”的阅读顺序，而不是只选单张视觉冲击最强的图。
- 输出理由要说明照片服务于哪个交付目的: 建筑形象、空间体验、材料细节、使用场景或环境关系。
- 若一张图透视非常端正但缺少空间气息，另一张略有瑕疵但能说明使用体验，应按项目目标判断，不机械追求完美线条。"#,
        ),
    ];

    definitions
        .into_iter()
        .map(
            |(prompt_pack_id, name, author, style_tags, scene_profile, shared_preference)| {
                let prompt_text = prompt_pack_content_json_from_input(shared_preference)
                    .expect("built-in prompt pack content should be valid JSON");
                PromptPack {
                    prompt_pack_id: prompt_pack_id.to_string(),
                    distribution_folder: "builtin".to_string(),
                    name: name.to_string(),
                    version: "builtin-v1".to_string(),
                    author: author.to_string(),
                    style_tags,
                    scene_profile,
                    schema: MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION.to_string(),
                    capabilities: default_prompt_pack_capabilities(),
                    built_in: true,
                    enabled: true,
                    prompt_hash: stable_prompt_hash(
                        MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION,
                        &prompt_text,
                    ),
                    prompt_text,
                    updated_at_ms: 0,
                }
            },
        )
        .collect()
}

pub(super) fn default_prompt_pack_capabilities() -> Vec<String> {
    vec![
        "single_evaluation".to_string(),
        "burst_selection".to_string(),
        "project_selection".to_string(),
    ]
}

pub(super) fn prompt_pack_sort_key(pack: &PromptPack) -> (u8, u8) {
    let built_in_order = match pack.prompt_pack_id.as_str() {
        "general-default" => 0,
        "documentary-integrity" => 1,
        "portrait-editorial" => 2,
        "portrait-lifestyle" => 3,
        "portrait-conservative" => 4,
        "landscape-fine-art" => 5,
        "landscape-technical" => 6,
        "wildlife-ethics" => 7,
        "action-sports-moment" => 8,
        "architecture-design" => 9,
        _ => 10,
    };
    (if pack.built_in { 0 } else { 1 }, built_in_order)
}

pub(super) fn prompt_packs_dir(state_dir: &Path) -> PathBuf {
    state_dir.join("prompt-packs")
}

pub(super) fn prompt_distribution_dir(state_dir: &Path, distribution_folder: &str) -> PathBuf {
    prompt_packs_dir(state_dir).join(normalized_distribution_folder(distribution_folder))
}

pub(super) fn prompt_pack_dir(
    state_dir: &Path,
    distribution_folder: &str,
    prompt_pack_id: &str,
) -> PathBuf {
    prompt_distribution_dir(state_dir, distribution_folder).join(stable_id_fragment(prompt_pack_id))
}

pub(super) fn unique_user_prompt_pack_id(state_dir: &Path, name: &str) -> Result<String> {
    let base = stable_id_fragment(name);
    let base = if base.is_empty() {
        "prompt-pack".to_string()
    } else {
        base
    };
    let builtin_ids = builtin_prompt_packs()
        .into_iter()
        .map(|pack| pack.prompt_pack_id)
        .collect::<HashSet<_>>();

    for index in 1..=999 {
        let candidate = if index == 1 {
            base.clone()
        } else {
            format!("{base}-{index}")
        };
        if builtin_ids.contains(&candidate) {
            continue;
        }
        if !prompt_pack_dir_exists_anywhere(state_dir, &candidate)? {
            return Ok(candidate);
        }
    }

    Err(crate::ImporterError::internal(
        "prompt pack name has too many duplicates",
    ))
}

pub(super) fn prompt_pack_dir_exists_anywhere(
    state_dir: &Path,
    prompt_pack_id: &str,
) -> Result<bool> {
    let root = prompt_packs_dir(state_dir);
    if !root.exists() {
        return Ok(false);
    }
    let prompt_pack_dir_name = stable_id_fragment(prompt_pack_id);
    for distribution_entry in fs::read_dir(root)? {
        let distribution_entry = distribution_entry?;
        if !distribution_entry.file_type()?.is_dir() {
            continue;
        }
        if distribution_entry
            .path()
            .join(&prompt_pack_dir_name)
            .exists()
        {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn load_user_prompt_packs(state_dir: &Path) -> Result<Vec<PromptPack>> {
    let root = prompt_packs_dir(state_dir);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut packs = Vec::new();
    for distribution_entry in fs::read_dir(root)? {
        let distribution_entry = distribution_entry?;
        if !distribution_entry.file_type()?.is_dir() {
            continue;
        }
        let distribution_folder =
            normalized_distribution_folder(&distribution_entry.file_name().to_string_lossy());
        for pack_entry in fs::read_dir(distribution_entry.path())? {
            let pack_entry = pack_entry?;
            if !pack_entry.file_type()?.is_dir() {
                continue;
            }
            let manifest_path = pack_entry.path().join("manifest.json");
            let prompt_path = pack_entry.path().join("PROMPT.md");
            if !manifest_path.exists() || !prompt_path.exists() {
                continue;
            }
            let mut pack: PromptPack =
                serde_json::from_str(&fs::read_to_string(&manifest_path)?)
                    .map_err(|error| crate::ImporterError::internal(error.to_string()))?;
            let prompt_markdown = fs::read_to_string(prompt_path)?;
            pack.distribution_folder = normalized_distribution_folder(&pack.distribution_folder);
            if pack.distribution_folder != distribution_folder {
                pack.distribution_folder = distribution_folder.clone();
            }
            pack.prompt_text = prompt_pack_content_json_from_markdown(&prompt_markdown)?;
            pack.prompt_hash =
                stable_prompt_hash(MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION, &pack.prompt_text);
            pack.schema = MODEL_EVALUATION_OUTPUT_SCHEMA_VERSION.to_string();
            pack.built_in = false;
            packs.push(pack);
        }
    }
    Ok(packs)
}

pub(super) fn save_user_prompt_pack(state_dir: &Path, pack: &PromptPack) -> Result<PromptPack> {
    if pack.built_in {
        return Err(crate::ImporterError::internal(
            "built-in prompt pack is read-only",
        ));
    }
    let distribution_folder = normalized_distribution_folder(&pack.distribution_folder);
    let dir = prompt_pack_dir(state_dir, &distribution_folder, &pack.prompt_pack_id);
    fs::create_dir_all(&dir)?;
    let prompt_markdown = prompt_pack_markdown_from_json(&pack.prompt_text)?;
    let mut manifest = pack.clone();
    manifest.distribution_folder = distribution_folder;
    manifest.prompt_text.clear();
    fs::write(
        dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)
            .map_err(|error| crate::ImporterError::internal(error.to_string()))?,
    )?;
    fs::write(dir.join("PROMPT.md"), prompt_markdown)?;
    Ok(pack.clone())
}

pub(super) fn normalized_prompt_pack_name(name: &str, fallback: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        format!("{fallback} 副本")
    } else {
        name.to_string()
    }
}

pub(super) fn normalized_distribution_folder(value: &str) -> String {
    let mut output = String::new();
    for character in value.trim().chars() {
        if character.is_alphanumeric() || character == '_' {
            output.push(character);
        } else if !output.ends_with('-') {
            output.push('-');
        }
    }
    let output = output.trim_matches(|character| character == '-' || character == '.');
    if output.is_empty() {
        "user".to_string()
    } else {
        output.to_string()
    }
}

pub(super) fn prompt_pack_content_json_from_input(value: &str) -> Result<String> {
    prompt_pack_content_json_from_markdown(value)
}

fn prompt_pack_content_json_from_markdown(value: &str) -> Result<String> {
    serde_json::to_string(&PromptPackContent::new(value.trim()))
        .map_err(|error| crate::ImporterError::internal(error.to_string()))
}

pub(super) fn prompt_pack_content_from_json(value: &str) -> Result<PromptPackContent> {
    serde_json::from_str(value).map_err(|error| {
        crate::ImporterError::internal(format!("invalid prompt pack content: {error}"))
    })
}

pub(super) fn prompt_pack_markdown_from_json(value: &str) -> Result<String> {
    Ok(prompt_pack_content_from_json(value)?.shared_preference)
}

pub(super) fn stable_id_fragment(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
        } else if !output.ends_with('-') {
            output.push('-');
        }
    }
    let output = output.trim_matches('-');
    if output.is_empty() {
        "id".to_string()
    } else {
        output.to_string()
    }
}
