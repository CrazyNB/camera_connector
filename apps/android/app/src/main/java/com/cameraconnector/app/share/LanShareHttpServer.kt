package com.cameraconnector.app.share

import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.GuestMark
import com.cameraconnector.app.core.ProjectAsset
import java.io.BufferedInputStream
import java.net.ServerSocket
import java.net.Socket
import java.net.URLDecoder
import java.net.URLEncoder
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.json.JSONArray
import org.json.JSONObject

const val LAN_PROJECT_SYNC_DISCOVERY_PORT = 48217

interface LanShareGateway {
    suspend fun loadAssets(token: String, offset: Int = 0, limit: Int = 2_000): List<ProjectAsset>
    suspend fun setGuestMark(token: String, groupId: String, guestMark: GuestMark?): GuestMark?
}

class CoreLanShareGateway(private val coreGateway: CoreGateway) : LanShareGateway {
    override suspend fun loadAssets(token: String, offset: Int, limit: Int): List<ProjectAsset> =
        coreGateway.loadLanShareAssets(token, offset, limit)

    override suspend fun setGuestMark(token: String, groupId: String, guestMark: GuestMark?): GuestMark? =
        coreGateway.setLanShareGuestMark(token, groupId, guestMark)
}

typealias LanSharePreviewLoader = suspend (token: String, groupId: String, fullQuality: Boolean) -> ByteArray?

data class LanShareDiscoveryInfo(
    val token: String,
    val projectName: String,
    val deviceLabel: String = "Android LAN Share",
    val platform: String = "android",
)

data class LanShareRequest(
    val method: String,
    val path: String,
    val body: String = "",
)

data class LanShareResponse(
    val status: Int,
    val contentType: String = "application/json; charset=utf-8",
    val body: ByteArray = ByteArray(0),
) {
    companion object {
        fun text(status: Int, contentType: String, body: String): LanShareResponse =
            LanShareResponse(status, contentType, body.toByteArray(Charsets.UTF_8))

        fun json(status: Int, value: JSONObject): LanShareResponse =
            text(status, "application/json; charset=utf-8", value.toString())
    }
}

class LanShareRouter(
    private val gateway: LanShareGateway,
    private val previewLoader: LanSharePreviewLoader,
    private val discoveryInfo: LanShareDiscoveryInfo? = null,
    private val projectSnapshotLoader: (suspend (token: String) -> List<ProjectAsset>)? = null,
) {
    suspend fun handle(request: LanShareRequest): LanShareResponse {
        val segments = request.path.substringBefore('?')
            .trim('/')
            .split('/')
            .filter { it.isNotBlank() }
        return when {
            request.method == "GET" && segments.size == 2 && segments[0] == "s" ->
                guestPage(segments[1])

            request.method == "GET" &&
                segments.size == 3 &&
                segments[0] == "api" &&
                segments[1] == "project-sync" &&
                segments[2] == "discovery" ->
                discovery()

            request.method == "GET" &&
                segments.size == 4 &&
                segments[0] == "api" &&
                segments[1] == "s" &&
                segments[3] == "assets" ->
                assetList(segments[2])

            request.method == "GET" &&
                segments.size == 4 &&
                segments[0] == "api" &&
                segments[1] == "s" &&
                segments[3] == "project-snapshot" ->
                projectSnapshot(segments[2])

            request.method == "GET" &&
                segments.size == 5 &&
                segments[0] == "api" &&
                segments[1] == "s" &&
                segments[3] == "preview" ->
                preview(segments[2], decodePathPart(segments[4]), fullQuality = false)

            request.method == "GET" &&
                segments.size == 5 &&
                segments[0] == "api" &&
                segments[1] == "s" &&
                segments[3] == "preview-full" ->
                preview(segments[2], decodePathPart(segments[4]), fullQuality = true)

            request.method == "PUT" &&
                segments.size == 6 &&
                segments[0] == "api" &&
                segments[1] == "s" &&
                segments[3] == "assets" &&
                segments[5] == "guest-mark" ->
                updateGuestMark(segments[2], decodePathPart(segments[4]), request.body)

            else -> LanShareResponse.json(404, JSONObject().put("error", "not_found"))
        }
    }

    private fun guestPage(token: String): LanShareResponse =
        LanShareResponse.text(
            200,
            "text/html; charset=utf-8",
            """
            <!doctype html>
            <html lang="zh-CN">
            <head>
              <meta charset="utf-8">
              <meta name="viewport" content="width=device-width, initial-scale=1">
              <title>多方筛选</title>
              <style>
                :root {
                  color-scheme: dark;
                  --bg: #061018;
                  --panel: #0b1b25;
                  --panel-2: #0e2330;
                  --border: #203546;
                  --text: #eaf6ff;
                  --muted: #82a0b2;
                  --blue: #00d5ff;
                  --green: #27e7a2;
                  --danger: #ff4d6d;
                }
                * { box-sizing: border-box; }
                body {
                  margin: 0;
                  min-height: 100vh;
                  background: var(--bg);
                  color: var(--text);
                  font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
                }
                header {
                  position: sticky;
                  top: 0;
                  z-index: 2;
                  padding: 14px 16px;
                  background: rgba(6, 16, 24, 0.94);
                  border-bottom: 1px solid var(--border);
                }
                h1 {
                  margin: 0;
                  font-size: 18px;
                  line-height: 1.25;
                }
                .subtle {
                  color: var(--muted);
                  font-size: 12px;
                  line-height: 1.4;
                }
                main {
                  width: min(1180px, 100%);
                  margin: 0 auto;
                  padding: 14px;
                }
                .toolbar {
                  display: flex;
                  flex-wrap: wrap;
                  gap: 10px 16px;
                  align-items: center;
                  margin-bottom: 14px;
                }
                .filter-group {
                  display: flex;
                  flex-wrap: wrap;
                  gap: 8px;
                }
                .filter-button {
                  padding: 8px 12px;
                  color: var(--muted);
                }
                .filter-button.active {
                  color: #031018;
                }
                .status {
                  padding: 28px 16px;
                  color: var(--muted);
                  text-align: center;
                }
                .lightbox {
                  position: fixed;
                  inset: 0;
                  z-index: 10;
                  display: none;
                  align-items: center;
                  justify-content: center;
                  background: rgba(0, 0, 0, 0.92);
                }
                .lightbox.open {
                  display: flex;
                }
                .lightbox img {
                  max-width: 100vw;
                  max-height: 100vh;
                  object-fit: contain;
                }
                .lightbox-close {
                  position: fixed;
                  top: 14px;
                  right: 14px;
                  width: 42px;
                  height: 42px;
                  padding: 0;
                  border-color: rgba(234, 246, 255, 0.34);
                  background: rgba(14, 35, 48, 0.82);
                  color: var(--text);
                  font-size: 22px;
                }
                .grid {
                  display: grid;
                  grid-template-columns: repeat(auto-fill, minmax(154px, 1fr));
                  gap: 12px;
                }
                .asset {
                  overflow: hidden;
                  border: 1px solid var(--border);
                  border-radius: 10px;
                  background: var(--panel);
                }
                .thumb {
                  display: block;
                  width: 100%;
                  aspect-ratio: 1 / 1;
                  object-fit: cover;
                  background: #02070b;
                  cursor: zoom-in;
                }
                .marks {
                  display: flex;
                  flex-wrap: nowrap;
                  gap: 4px;
                  min-height: 28px;
                  overflow: hidden;
                  padding: 6px 8px;
                }
                .chip {
                  border: 1px solid var(--border);
                  border-radius: 999px;
                  flex: 0 1 auto;
                  max-width: 50%;
                  overflow: hidden;
                  padding: 2px 6px;
                  color: var(--muted);
                  font-size: 10px;
                  line-height: 1;
                  text-overflow: ellipsis;
                  white-space: nowrap;
                }
                .chip.photographer {
                  border-color: rgba(0, 213, 255, 0.58);
                  color: var(--blue);
                }
                .chip.guest {
                  border-color: rgba(39, 231, 162, 0.58);
                  color: var(--green);
                }
                .actions {
                  display: flex;
                  gap: 5px;
                  padding: 6px 8px 8px;
                  border-top: 1px solid rgba(32, 53, 70, 0.72);
                }
                button {
                  flex: 1 1 0;
                  min-width: 0;
                  border: 1px solid var(--border);
                  border-radius: 999px;
                  background: var(--panel-2);
                  color: var(--text);
                  font: inherit;
                  font-size: 11px;
                  font-weight: 700;
                  line-height: 1;
                  padding: 6px 6px;
                  white-space: nowrap;
                }
                button.active {
                  border-color: transparent;
                  background: var(--blue);
                  color: #031018;
                }
                button.reject.active {
                  background: var(--danger);
                  color: #fff;
                }
                button:disabled {
                  opacity: 0.56;
                }
                @media (max-width: 520px) {
                  main { padding: 10px; }
                  .toolbar { gap: 8px; }
                  .grid {
                    grid-template-columns: repeat(2, minmax(0, 1fr));
                    gap: 10px;
                  }
                }
              </style>
            </head>
            <body data-token="${htmlAttrEscape(token)}">
              <header>
                <h1>多方筛选</h1>
                <div id="summary" class="subtle">正在载入照片</div>
              </header>
              <main>
                <div id="controls" class="toolbar"></div>
                <section id="app"><div class="status">正在载入照片...</div></section>
              </main>
              <div id="lightbox" class="lightbox" role="dialog" aria-modal="true">
                <button id="lightboxClose" class="lightbox-close" type="button" aria-label="关闭">×</button>
                <img id="lightboxImage" alt="">
              </div>
              <script>
                (function () {
                  var token = document.body.dataset.token || "";
                  var app = document.getElementById("app");
                  var controls = document.getElementById("controls");
                  var summary = document.getElementById("summary");
                  var lightbox = document.getElementById("lightbox");
                  var lightboxImage = document.getElementById("lightboxImage");
                  var lightboxClose = document.getElementById("lightboxClose");
                  var assets = [];
                  var guestFilter = "all";
                  var minScore = null;

                  function text(value) {
                    return value == null ? "" : String(value);
                  }

                  function markLabel(mark) {
                    if (mark === "favorite") return "收藏";
                    if (mark === "marked") return "标记";
                    if (mark === "reject") return "删除";
                    return "未标记";
                  }

                  function render() {
                    var visible = visibleAssets();
                    renderControls();
                    summary.textContent = visible.length === assets.length
                      ? assets.length + " 张照片"
                      : visible.length + " / " + assets.length + " 张照片";
                    if (!visible.length) {
                      app.innerHTML = '<div class="status">当前没有可筛选照片</div>';
                      return;
                    }
                    var grid = document.createElement("section");
                    grid.className = "grid";
                    visible.forEach(function (asset) {
                      grid.appendChild(assetCard(asset));
                    });
                    app.replaceChildren(grid);
                  }

                  function visibleAssets() {
                    return assets.filter(function (asset) {
                      if (guestFilter !== "all") {
                        if (guestFilter === "none" && asset.guest_mark) return false;
                        if (guestFilter !== "none" && asset.guest_mark !== guestFilter) return false;
                      }
                      if (minScore != null) {
                        var score = Number(asset.model_score);
                        if (!Number.isFinite(score) || score < minScore) return false;
                      }
                      return true;
                    });
                  }

                  function renderControls() {
                    var marks = document.createElement("div");
                    marks.className = "filter-group";
                    [
                      ["all", "全部"],
                      ["favorite", "收藏"],
                      ["marked", "标记"],
                      ["reject", "删除"],
                      ["none", "未标记"]
                    ].forEach(function (item) {
                      marks.appendChild(filterButton(item[1], guestFilter === item[0], function () {
                        guestFilter = item[0];
                        render();
                      }));
                    });

                    var scores = document.createElement("div");
                    scores.className = "filter-group";
                    [
                      [null, "不限"],
                      [60, "≥60"],
                      [70, "≥70"],
                      [80, "≥80"]
                    ].forEach(function (item) {
                      scores.appendChild(filterButton(item[1], minScore === item[0], function () {
                        minScore = item[0];
                        render();
                      }));
                    });

                    controls.replaceChildren(marks, scores);
                  }

                  function filterButton(label, active, onClick) {
                    var button = document.createElement("button");
                    button.type = "button";
                    button.className = active ? "filter-button active" : "filter-button";
                    button.textContent = label;
                    button.addEventListener("click", onClick);
                    return button;
                  }

                  function assetCard(asset) {
                    var article = document.createElement("article");
                    article.className = "asset";

                    var img = document.createElement("img");
                    img.className = "thumb";
                    img.loading = "lazy";
                    img.alt = text(asset.display_path);
                    img.src = text(asset.preview_url);
                    img.addEventListener("click", function () {
                      openLightbox(asset);
                    });
                    article.appendChild(img);

                    var marks = document.createElement("div");
                    marks.className = "marks";
                    if (asset.user_marks && asset.user_marks.favorite) {
                      marks.appendChild(chip("收藏", "photographer"));
                    }
                    if (asset.user_marks && asset.user_marks.marked) {
                      marks.appendChild(chip("标记", "photographer"));
                    }
                    if (asset.guest_mark) {
                      marks.appendChild(chip(markLabel(asset.guest_mark), "guest"));
                    }
                    article.appendChild(marks);

                    var actions = document.createElement("div");
                    actions.className = "actions";
                    actions.appendChild(markButton(asset, "favorite", "收藏"));
                    actions.appendChild(markButton(asset, "marked", "标记"));
                    actions.appendChild(markButton(asset, "reject", "删除", "reject"));
                    article.appendChild(actions);

                    return article;
                  }

                  function chip(label, tone) {
                    var node = document.createElement("span");
                    node.className = tone ? "chip " + tone : "chip";
                    node.textContent = label;
                    return node;
                  }

                  function markButton(asset, mark, label, extraClass) {
                    var button = document.createElement("button");
                    button.type = "button";
                    button.textContent = label;
                    if (asset.guest_mark === mark) {
                      button.className = "active" + (extraClass ? " " + extraClass : "");
                    } else if (extraClass) {
                      button.className = extraClass;
                    }
                    button.addEventListener("click", function () {
                      setMark(asset, asset.guest_mark === mark ? null : mark, button);
                    });
                    return button;
                  }

                  function setMark(asset, mark, button) {
                    button.disabled = true;
                    fetch("/api/s/" + encodeURIComponent(token) + "/assets/" + encodeURIComponent(asset.id) + "/guest-mark", {
                      method: "PUT",
                      headers: { "Content-Type": "application/json" },
                      body: JSON.stringify({ guest_mark: mark })
                    })
                      .then(function (response) {
                        if (!response.ok) throw new Error("mark_failed");
                        return response.json();
                      })
                      .then(function (payload) {
                        asset.guest_mark = payload.guest_mark || null;
                        render();
                      })
                      .catch(function () {
                        summary.textContent = "标记失败，请刷新后重试";
                        button.disabled = false;
                      });
                  }

                  function openLightbox(asset) {
                    lightboxImage.removeAttribute("src");
                    lightboxImage.alt = text(asset.display_path);
                    lightboxImage.src = text(asset.full_preview_url || asset.preview_url);
                    lightbox.classList.add("open");
                  }

                  function closeLightbox() {
                    lightbox.classList.remove("open");
                    lightboxImage.removeAttribute("src");
                  }

                  lightbox.addEventListener("click", function (event) {
                    if (event.target === lightbox) {
                      closeLightbox();
                    }
                  });
                  lightboxClose.addEventListener("click", closeLightbox);
                  document.addEventListener("keydown", function (event) {
                    if (event.key === "Escape" && lightbox.classList.contains("open")) {
                      closeLightbox();
                    }
                  });

                  fetch("/api/s/" + encodeURIComponent(token) + "/assets")
                    .then(function (response) {
                      if (!response.ok) throw new Error("assets_failed");
                      return response.json();
                    })
                    .then(function (payload) {
                      assets = Array.isArray(payload.assets) ? payload.assets : [];
                      render();
                    })
                    .catch(function () {
                      summary.textContent = "载入失败";
                      app.innerHTML = '<div class="status">无法载入共享照片，请确认共享仍在开启。</div>';
                    });
                })();
              </script>
            </body>
            </html>
            """.trimIndent(),
        )

    private fun discovery(): LanShareResponse {
        val info = discoveryInfo
            ?: return LanShareResponse.json(404, JSONObject().put("error", "discovery_not_available"))
        return LanShareResponse.json(
            200,
            JSONObject()
                .put("device_label", info.deviceLabel)
                .put("platform", info.platform)
                .put("project_name", info.projectName)
                .put("snapshot_path", "/api/s/${encodePathPart(info.token)}/project-snapshot"),
        )
    }

    private suspend fun assetList(token: String): LanShareResponse {
        val assets = gateway.loadAssets(token)
        return LanShareResponse.json(
            200,
            JSONObject().put(
                "assets",
                JSONArray().apply {
                    assets.forEach { put(it.toLanShareJson(token)) }
                },
            ),
        )
    }

    private suspend fun projectSnapshot(token: String): LanShareResponse {
        val assets = projectSnapshotLoader?.invoke(token) ?: gateway.loadAssets(token)
        return LanShareResponse.json(
            200,
            assets.toProjectSyncSnapshotJson(token, System.currentTimeMillis()),
        )
    }

    private suspend fun preview(token: String, groupId: String, fullQuality: Boolean): LanShareResponse {
        val bytes = previewLoader(token, groupId, fullQuality)
            ?: return LanShareResponse.json(404, JSONObject().put("error", "preview_not_found"))
        return LanShareResponse(200, "image/jpeg", bytes)
    }

    private suspend fun updateGuestMark(token: String, groupId: String, body: String): LanShareResponse {
        val payload = body.takeIf { it.isNotBlank() }?.let(::JSONObject) ?: JSONObject()
        val nextMark = if (payload.has("guest_mark") && !payload.isNull("guest_mark")) {
            guestMarkFromWire(payload.optString("guest_mark"))
                ?: return LanShareResponse.json(400, JSONObject().put("error", "invalid_guest_mark"))
        } else {
            null
        }
        val saved = gateway.setGuestMark(token, groupId, nextMark)
        return LanShareResponse.json(
            200,
            JSONObject().put("guest_mark", saved?.wireName ?: JSONObject.NULL),
        )
    }
}

class LanShareHttpServer(
    private val router: LanShareRouter,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : AutoCloseable {
    private var serverSocket: ServerSocket? = null
    private var acceptJob: Job? = null

    val port: Int
        get() = serverSocket?.localPort ?: 0

    fun start(port: Int = 0): Int {
        check(serverSocket == null) { "LAN share server is already running" }
        val socket = ServerSocket(port)
        serverSocket = socket
        acceptJob = scope.launch {
            while (isActive && !socket.isClosed) {
                runCatching { socket.accept() }
                    .onSuccess { client -> launch { handleClient(client) } }
            }
        }
        return socket.localPort
    }

    fun stop() {
        acceptJob?.cancel()
        acceptJob = null
        serverSocket?.close()
        serverSocket = null
    }

    override fun close() {
        stop()
        scope.cancel()
    }

    private fun handleClient(client: Socket) {
        client.use { socket ->
            val request = readHttpRequest(socket) ?: return
            val response = runBlocking { router.handle(request) }
            socket.getOutputStream().use { output ->
                output.write(httpResponseBytes(response))
                output.flush()
            }
        }
    }
}

private fun ProjectAsset.toLanShareJson(token: String): JSONObject =
    (id.ifBlank { displayPath }).let { shareId ->
    JSONObject()
        .put("id", shareId)
        .put("display_path", displayPath)
        .put("format", format)
        .put("model_score", modelScore ?: JSONObject.NULL)
        .put("preview_url", "/api/s/${encodePathPart(token)}/preview/${encodePathPart(shareId)}")
        .put("full_preview_url", "/api/s/${encodePathPart(token)}/preview-full/${encodePathPart(shareId)}")
        .put("guest_mark", guestMark?.wireName ?: JSONObject.NULL)
        .put(
            "user_marks",
            JSONObject()
                .put("favorite", userMarks.favorite)
                .put("marked", userMarks.marked),
        )
    }

private fun List<ProjectAsset>.toProjectSyncSnapshotJson(token: String, exportedAtMs: Long): JSONObject {
    val snapshotAssets = JSONArray()
    val snapshotGroups = JSONArray()
    val userMarks = JSONArray()
    val modelEvaluations = JSONArray()
    val candidateGroupIds = JSONArray()
    val selectedGroupIds = JSONArray()

    forEach { asset ->
        val groupId = asset.id.ifBlank { asset.groupKey.ifBlank { asset.displayPath } }
        val assetId = "$groupId:primary"
        val path = asset.originalPath?.takeIf { it.isNotBlank() } ?: asset.displayPath
        val filename = lastPathPart(path).ifBlank { lastPathPart(asset.displayPath).ifBlank { groupId } }
        val normalizedStem = normalizedStem(filename).ifBlank { normalizedStem(groupId) }
        val receivedAtMs = asset.receivedAt.toLongOrNull()
        val sourceIdentity = asset.displaySource ?: asset.username

        snapshotAssets.put(
            JSONObject()
                .put("asset_id", assetId)
                .put("group_id", groupId)
                .put("original_filename", filename)
                .put("final_filename", lastPathPart(asset.displayPath).ifBlank { filename })
                .put("normalized_stem", normalizedStem)
                .put("original_path", path)
                .put("original_parent_path", parentPath(path))
                .put("format", asset.format.lowercase())
                .put("size_bytes", asset.sizeBytes ?: 0L)
                .put("capture_at_ms", JSONObject.NULL)
                .put("received_at_ms", receivedAtMs ?: JSONObject.NULL)
                .put("source_identity", sourceIdentity ?: JSONObject.NULL),
        )
        snapshotGroups.put(
            JSONObject()
                .put("group_id", groupId)
                .put("display_key", asset.groupKey.ifBlank { normalizedStem })
                .put("source_identity", sourceIdentity ?: JSONObject.NULL)
                .put("original_parent_path", parentPath(path))
                .put("member_asset_ids", JSONArray().put(assetId))
                .put("primary_asset_id", assetId)
                .put("preview_asset_id", assetId)
                .put("has_raw", asset.hasRaw)
                .put("has_jpeg", asset.hasJpeg)
                .put("has_video", asset.hasVideo),
        )

        if (asset.userMarks.favorite || asset.userMarks.marked) {
            userMarks.put(
                JSONObject()
                    .put("group_id", groupId)
                    .put("favorite", asset.userMarks.favorite)
                    .put("marked", asset.userMarks.marked),
            )
        }

        if (asset.modelScore != null) {
            modelEvaluations.put(
                JSONObject()
                    .put("evaluation_id", "lan-share:$groupId")
                    .put("group_id", groupId)
                    .put("evaluator_version", asset.modelEvaluatorKind ?: "android-lan-share")
                    .put("status", asset.modelStatus ?: "ready")
                    .put("score", asset.modelScore)
                    .put("tier", asset.modelTier ?: "")
                    .put("selectable", asset.isModelSelect)
                    .put("summary", asset.modelSummary ?: "")
                    .put("strengths", JSONArray())
                    .put("weaknesses", JSONArray())
                    .put("technical_warnings", JSONArray())
                    .put("prompt_pack_id", JSONObject.NULL)
                    .put("prompt_pack_version", JSONObject.NULL)
                    .put("prompt_hash", JSONObject.NULL)
                    .put("created_at_ms", receivedAtMs ?: exportedAtMs)
                    .put("updated_at_ms", receivedAtMs ?: exportedAtMs),
            )
        }

        candidateGroupIds.put(groupId)
        if (asset.isModelSelect) {
            selectedGroupIds.put(groupId)
        }
    }

    val recommendations = if (selectedGroupIds.length() > 0) {
        JSONArray().put(
            JSONObject()
                .put("recommendation_id", "lan-share:model-select")
                .put("scope", "project")
                .put("subject_group_id", JSONObject.NULL)
                .put("selected_group_ids", selectedGroupIds)
                .put("candidate_group_ids", candidateGroupIds)
                .put("rejected_group_ids", JSONArray())
                .put("status", "ready")
                .put("confidence", 1.0)
                .put("reason", "Imported Android model selections")
                .put("created_at_ms", exportedAtMs)
                .put("updated_at_ms", exportedAtMs),
        )
    } else {
        JSONArray()
    }

    return JSONObject()
        .put("schema_version", 1)
        .put(
            "source_device",
            JSONObject()
                .put("device_id", "lan-share:$token")
                .put("device_label", "Android LAN Share")
                .put("platform", "android"),
        )
        .put(
            "project",
            JSONObject()
                .put("project_id", "lan-share:$token")
                .put("name", "Android LAN Share")
                .put("exported_at_ms", exportedAtMs),
        )
        .put("assets", snapshotAssets)
        .put("groups", snapshotGroups)
        .put("model_evaluations", modelEvaluations)
        .put("selection_recommendations", recommendations)
        .put("user_marks", userMarks)
}

private fun lastPathPart(value: String): String =
    value.trim().replace('\\', '/').substringAfterLast('/')

private fun parentPath(value: String): Any {
    val normalized = value.trim().replace('\\', '/')
    val parent = normalized.substringBeforeLast('/', missingDelimiterValue = "")
    return parent.takeIf { it.isNotBlank() } ?: JSONObject.NULL
}

private fun normalizedStem(value: String): String =
    lastPathPart(value).substringBeforeLast('.', missingDelimiterValue = lastPathPart(value)).lowercase()

private fun guestMarkFromWire(value: String): GuestMark? =
    when (value.trim().lowercase()) {
        GuestMark.Favorite.wireName -> GuestMark.Favorite
        GuestMark.Marked.wireName -> GuestMark.Marked
        GuestMark.Reject.wireName -> GuestMark.Reject
        else -> null
    }

private fun readHttpRequest(socket: Socket): LanShareRequest? {
    val input = BufferedInputStream(socket.getInputStream())
    val headerBytes = mutableListOf<Byte>()
    while (true) {
        val next = input.read()
        if (next < 0) return null
        headerBytes.add(next.toByte())
        val size = headerBytes.size
        if (
            size >= 4 &&
            headerBytes[size - 4] == '\r'.code.toByte() &&
            headerBytes[size - 3] == '\n'.code.toByte() &&
            headerBytes[size - 2] == '\r'.code.toByte() &&
            headerBytes[size - 1] == '\n'.code.toByte()
        ) {
            break
        }
    }
    val headers = headerBytes.toByteArray().toString(Charsets.ISO_8859_1)
    val lines = headers.split("\r\n")
    val requestLine = lines.firstOrNull()?.split(' ') ?: return null
    val contentLength = lines
        .firstOrNull { it.startsWith("content-length:", ignoreCase = true) }
        ?.substringAfter(':')
        ?.trim()
        ?.toIntOrNull()
        ?.coerceAtLeast(0)
        ?: 0
    val body = ByteArray(contentLength)
    var read = 0
    while (read < contentLength) {
        val count = input.read(body, read, contentLength - read)
        if (count < 0) break
        read += count
    }
    return LanShareRequest(
        method = requestLine.getOrNull(0).orEmpty(),
        path = requestLine.getOrNull(1).orEmpty(),
        body = body.copyOf(read).toString(Charsets.UTF_8),
    )
}

private fun httpResponseBytes(response: LanShareResponse): ByteArray {
    val statusText = when (response.status) {
        200 -> "OK"
        400 -> "Bad Request"
        404 -> "Not Found"
        else -> "Error"
    }
    val header = buildString {
        append("HTTP/1.1 ${response.status} $statusText\r\n")
        append("Connection: close\r\n")
        append("Content-Type: ${response.contentType}\r\n")
        append("Content-Length: ${response.body.size}\r\n")
        append("\r\n")
    }.toByteArray(Charsets.UTF_8)
    return header + response.body
}

private fun decodePathPart(value: String): String =
    URLDecoder.decode(value, Charsets.UTF_8.name())

private fun encodePathPart(value: String): String =
    URLEncoder.encode(value, Charsets.UTF_8.name()).replace("+", "%20")

private fun htmlAttrEscape(value: String): String =
    value
        .replace("&", "&amp;")
        .replace("\"", "&quot;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
