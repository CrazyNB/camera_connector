package com.cameraconnector.app.ui

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri
import java.io.File
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.PhotoLibrary
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material.icons.outlined.SyncAlt
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Shapes
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.cameraconnector.app.core.CoreGateway
import com.cameraconnector.app.core.DashboardState
import com.cameraconnector.app.core.DeviceAccount
import com.cameraconnector.app.core.InboxAsset
import com.cameraconnector.app.core.ReceiverSettings
import com.cameraconnector.app.core.ReceiverState
import com.cameraconnector.app.storage.AndroidStorageGateway
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch

@Composable
fun CameraConnectorApp(
    coreGateway: CoreGateway,
    storageGateway: AndroidStorageGateway,
    notificationPermissionRequired: Boolean,
    notificationPermissionGranted: Flow<Boolean>,
    selectedInboxLabel: Flow<String?>,
    onRequestNotificationPermission: () -> Unit,
    onChooseInboxDirectory: () -> Unit,
) {
    val dashboard by coreGateway.observeDashboard().collectAsState(
        initial = DashboardState(
            receiver = ReceiverState(
                running = false,
                phase = "Unknown",
                protocol = "FTP",
                authMode = "Unknown",
                accountCount = 0,
                host = DEFAULT_LISTEN_HOST,
                port = 2121,
                outputLabel = "未配置",
                message = null,
            ),
            accounts = emptyList(),
            inbox = emptyList(),
            transfers = emptyList(),
        ),
    )
    val notificationsGranted by notificationPermissionGranted.collectAsState(initial = true)
    val selectedInbox by selectedInboxLabel.collectAsState(initial = null)
    val scope = rememberCoroutineScope()
    var tab by remember { mutableStateOf(MainTab.Overview) }
    var settingsOpen by remember { mutableStateOf(false) }
    var accountDetail by remember { mutableStateOf<DeviceAccount?>(null) }
    var addingAccount by remember { mutableStateOf(false) }
    var actionError by remember { mutableStateOf<String?>(null) }
    var actionInFlight by remember { mutableStateOf<String?>(null) }
    var inboxGridColumnCount by rememberSaveable { mutableStateOf(storageGateway.inboxGridColumnCount()) }

    fun runAction(actionName: String, action: suspend () -> Unit) {
        scope.launch {
            actionInFlight = actionName
            try {
                action()
                actionError = null
            } catch (error: Throwable) {
                actionError = error.message ?: error::class.java.simpleName
            } finally {
                actionInFlight = null
            }
        }
    }

    MaterialTheme(
        colorScheme = elementColorScheme,
        shapes = Shapes(small = elementShape, medium = elementShape, large = elementShape),
    ) {
        Surface(
            modifier = Modifier.fillMaxSize(),
            color = MaterialTheme.colorScheme.background,
        ) {
            Scaffold(
                containerColor = MaterialTheme.colorScheme.background,
                bottomBar = {
                    if (!settingsOpen) {
                        NavigationBar(containerColor = MaterialTheme.colorScheme.surface) {
                            MainTab.entries.forEach { item ->
                                NavigationBarItem(
                                    selected = tab == item,
                                    onClick = { tab = item },
                                    label = { Text(item.label) },
                                    icon = { Icon(item.icon, contentDescription = item.label) },
                                    colors = NavigationBarItemDefaults.colors(
                                        selectedIconColor = MaterialTheme.colorScheme.primary,
                                        selectedTextColor = MaterialTheme.colorScheme.primary,
                                        indicatorColor = ElementBlueSoft,
                                    ),
                                )
                            }
                        }
                    }
                },
            ) { padding ->
                if (settingsOpen) {
                    val selectedAccount = accountDetail
                    if (selectedAccount != null || addingAccount) {
                        AccountDetailScreen(
                            account = selectedAccount,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            onBack = {
                                accountDetail = null
                                addingAccount = false
                            },
                            onSaveDeviceAccount = { account, password ->
                                runAction("正在保存账号") {
                                    coreGateway.saveDeviceAccount(account, password)
                                }
                            },
                            onDeleteDeviceAccount = { username ->
                                runAction("正在删除账号") {
                                    coreGateway.removeDeviceAccount(username)
                                }
                                accountDetail = null
                                addingAccount = false
                            },
                            modifier = Modifier.padding(padding),
                        )
                    } else {
                        SettingsScreen(
                            dashboard = dashboard,
                            notificationPermissionRequired = notificationPermissionRequired,
                            notificationPermissionGranted = notificationsGranted,
                            onRequestNotificationPermission = onRequestNotificationPermission,
                            actionError = actionError,
                            actionInFlight = actionInFlight,
                            onClearActionError = { actionError = null },
                            selectedInboxLabel = selectedInbox,
                            onChooseInboxDirectory = onChooseInboxDirectory,
                            onCloseSettings = {
                                settingsOpen = false
                                accountDetail = null
                                addingAccount = false
                            },
                            onOpenAccount = {
                                accountDetail = it
                                addingAccount = false
                            },
                            onAddAccount = {
                                accountDetail = null
                                addingAccount = true
                            },
                            modifier = Modifier.padding(padding),
                        )
                    }
                } else when (tab) {
                    MainTab.Overview -> OverviewScreen(
                        dashboard = dashboard,
                        notificationPermissionGranted = notificationsGranted,
                        actionError = actionError,
                        actionInFlight = actionInFlight,
                        onClearActionError = { actionError = null },
                        onOpenSettings = { settingsOpen = true },
                        onToggleReceiver = {
                            if (dashboard.receiver.running) {
                                runAction("正在停止接收服务") { coreGateway.stopReceiver() }
                            } else {
                                runAction("正在启动接收服务") { coreGateway.startReceiver() }
                            }
                        },
                        onSaveReceiverSettings = { settings ->
                            runAction("正在保存接收设置") {
                                coreGateway.saveReceiverSettings(settings)
                            }
                        },
                        modifier = Modifier.padding(padding),
                    )

                    MainTab.Inbox -> InboxScreen(
                        dashboard = dashboard,
                        gridColumnCount = inboxGridColumnCount,
                        onGridColumnCountChange = { count ->
                            inboxGridColumnCount = count
                            storageGateway.persistInboxGridColumnCount(count)
                        },
                        modifier = Modifier.padding(padding),
                    )

                    MainTab.Transfers -> TransfersScreen(
                        dashboard = dashboard,
                        modifier = Modifier.padding(padding),
                    )
                }
            }
        }
    }
}

private enum class MainTab(val label: String, val icon: ImageVector) {
    Overview("总览", Icons.Outlined.Home),
    Inbox("收件箱", Icons.Outlined.PhotoLibrary),
    Transfers("传输", Icons.Outlined.SyncAlt),
}

private enum class InboxFilter(val label: String) {
    All("全部文件"),
    Raw("RAW"),
    Jpeg("JPEG"),
    Video("视频"),
}

private val ElementBlue = Color(0xFF409EFF)
private val ElementBlueSoft = Color(0xFFEcf5ff)
private val ElementSuccess = Color(0xFF67C23A)
private val ElementWarning = Color(0xFFE6A23C)
private val ElementDanger = Color(0xFFF56C6C)
private val ElementInfo = Color(0xFF909399)
private val ElementBorder = Color(0xFFDCDFE6)
private const val DEFAULT_LISTEN_HOST = "192.168.137.1"

private val elementColorScheme = lightColorScheme(
    primary = ElementBlue,
    secondary = ElementInfo,
    tertiary = ElementSuccess,
    error = ElementDanger,
    background = Color(0xFFF5F7FA),
    surface = Color.White,
    outline = ElementBorder,
    onSurface = Color(0xFF303133),
    onSurfaceVariant = Color(0xFF606266),
)

private val elementShape = RoundedCornerShape(4.dp)

@Composable
private fun ElementCard(
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    Card(
        modifier = modifier,
        shape = elementShape,
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
        elevation = CardDefaults.cardElevation(defaultElevation = 0.dp),
    ) {
        content()
    }
}

@Composable
private fun ElementTag(text: String, color: Color) {
    Surface(
        color = color.copy(alpha = 0.12f),
        contentColor = color,
        shape = elementShape,
        border = BorderStroke(1.dp, color.copy(alpha = 0.35f)),
    ) {
        Text(
            text,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.SemiBold,
        )
    }
}

@Composable
private fun OverviewScreen(
    dashboard: DashboardState,
    notificationPermissionGranted: Boolean,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onOpenSettings: () -> Unit,
    onToggleReceiver: () -> Unit,
    onSaveReceiverSettings: (ReceiverSettings) -> Unit,
    modifier: Modifier = Modifier,
) {
    var protocol by remember(dashboard.receiver.protocol) {
        mutableStateOf(dashboard.receiver.protocol.ifBlank { "FTP" })
    }
    val displayHost = normalizeListenHost(dashboard.receiver.host)
    var hostInput by remember(displayHost) {
        mutableStateOf(displayHost)
    }
    var portInput by remember(dashboard.receiver.port) {
        mutableStateOf(dashboard.receiver.port.takeIf { it in 1..65_535 }?.toString() ?: "2121")
    }
    val port = portInput.toIntOrNull()
    val receiverSettingsValid = hostInput.trim().isNotBlank() && port in 1..65_535
    val actionsEnabled = actionInFlight == null
    val onlineConnections = dashboard.accounts.sumOf { it.activeConnections }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column {
                    Text("Camera Connector", style = MaterialTheme.typography.headlineMedium)
                    Spacer(Modifier.height(4.dp))
                    Text("接收服务控制", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                IconButton(onClick = onOpenSettings) {
                    Icon(Icons.Outlined.Settings, contentDescription = "设置")
                }
            }
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        actionInFlight?.let { action ->
            item { ProcessingCard(action) }
        }

        item {
            ReceiverHeroControl(
                running = dashboard.receiver.running,
                phase = dashboard.receiver.phase,
                endpoint = "${dashboard.receiver.protocol} $displayHost:${dashboard.receiver.port}",
                onlineConnections = onlineConnections,
                accountCount = dashboard.accounts.size,
                message = dashboard.receiver.message,
                enabled = actionsEnabled && (dashboard.receiver.running || notificationPermissionGranted),
                onToggleReceiver = onToggleReceiver,
                modifier = Modifier.fillMaxWidth(),
            )
        }

        item {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    Text("接收设置", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(12.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        ProtocolSegment(
                            label = "FTP",
                            selected = protocol == "FTP",
                            enabled = actionsEnabled && !dashboard.receiver.running,
                            onClick = { protocol = "FTP" },
                            modifier = Modifier.weight(1f),
                        )
                        ProtocolSegment(
                            label = "SFTP",
                            selected = protocol == "SFTP",
                            enabled = actionsEnabled && !dashboard.receiver.running,
                            onClick = { protocol = "SFTP" },
                            modifier = Modifier.weight(1f),
                        )
                    }
                    Spacer(Modifier.height(12.dp))
                    OutlinedTextField(
                        value = hostInput,
                        onValueChange = { hostInput = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("监听地址") },
                        singleLine = true,
                        enabled = actionsEnabled && !dashboard.receiver.running,
                    )
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(
                        value = portInput,
                        onValueChange = { portInput = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("统一端口") },
                        singleLine = true,
                        enabled = actionsEnabled && !dashboard.receiver.running,
                    )
                    if (dashboard.receiver.running) {
                        Spacer(Modifier.height(8.dp))
                        Text(
                            "修改设置前请先停止接收服务。",
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    Spacer(Modifier.height(12.dp))
                    Button(
                        onClick = {
                            val cleanPort = port ?: 2121
                            onSaveReceiverSettings(
                                ReceiverSettings(
                                    protocol = protocol,
                                    host = normalizeListenHost(hostInput.trim()),
                                    ftpPort = cleanPort,
                                    sftpPort = cleanPort,
                                    outputLabel = dashboard.receiver.outputLabel,
                                ),
                            )
                        },
                        enabled = actionsEnabled && !dashboard.receiver.running && receiverSettingsValid,
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        Text("保存接收设置")
                    }
                }
            }
        }
    }
}

@Composable
private fun ReceiverHeroControl(
    running: Boolean,
    phase: String,
    endpoint: String,
    onlineConnections: Int,
    accountCount: Int,
    message: String?,
    enabled: Boolean,
    onToggleReceiver: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier.padding(vertical = 8.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(
            if (running) "接收服务运行中" else "接收服务已停止",
            style = MaterialTheme.typography.titleMedium,
        )
        Spacer(Modifier.height(6.dp))
        Text(endpoint, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Spacer(Modifier.height(22.dp))
        PowerButton(
            running = running,
            enabled = enabled,
            onClick = onToggleReceiver,
        )
        Spacer(Modifier.height(18.dp))
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            ElementTag(
                text = receiverPhaseLabel(phase),
                color = if (running) ElementSuccess else ElementInfo,
            )
            ElementTag(
                text = if (onlineConnections > 0) "在线连接 $onlineConnections" else "未连接",
                color = if (onlineConnections > 0) ElementSuccess else ElementInfo,
            )
            ElementTag(text = "已配置账号 $accountCount", color = ElementBlue)
        }
        message?.let {
            Spacer(Modifier.height(8.dp))
            Text(it, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
private fun PowerButton(
    running: Boolean,
    enabled: Boolean,
    onClick: () -> Unit,
) {
    Button(
        onClick = onClick,
        enabled = enabled,
        modifier = Modifier.size(168.dp),
        shape = CircleShape,
        colors = ButtonDefaults.buttonColors(
            containerColor = if (running) ElementDanger else ElementBlue,
            disabledContainerColor = ElementInfo.copy(alpha = 0.35f),
        ),
        contentPadding = PaddingValues(0.dp),
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(
                if (running) "停止" else "启动",
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
            )
            Spacer(Modifier.height(4.dp))
            Text(if (running) "接收服务" else "开始接收")
        }
    }
}

@Composable
private fun ProtocolSegment(
    label: String,
    selected: Boolean,
    enabled: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    OutlinedButton(
        onClick = onClick,
        enabled = enabled,
        modifier = modifier,
        border = BorderStroke(1.dp, if (selected) ElementBlue else ElementBorder),
        colors = ButtonDefaults.outlinedButtonColors(
            containerColor = if (selected) ElementBlue else Color.White,
            contentColor = if (selected) Color.White else MaterialTheme.colorScheme.onSurface,
            disabledContainerColor = if (selected) ElementBlue.copy(alpha = 0.55f) else Color.White,
            disabledContentColor = if (selected) Color.White else ElementInfo,
        ),
        shape = elementShape,
    ) {
        Text(label)
    }
}

@Composable
private fun SettingsScreen(
    dashboard: DashboardState,
    notificationPermissionRequired: Boolean,
    notificationPermissionGranted: Boolean,
    onRequestNotificationPermission: () -> Unit,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    selectedInboxLabel: String?,
    onChooseInboxDirectory: () -> Unit,
    onCloseSettings: () -> Unit,
    onOpenAccount: (DeviceAccount) -> Unit,
    onAddAccount: () -> Unit,
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            HeaderWithBack(
                title = "设置",
                subtitle = "账号、目录、通知权限",
                onBack = onCloseSettings,
            )
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        actionInFlight?.let { action ->
            item { ProcessingCard(action) }
        }

        if (notificationPermissionRequired && !notificationPermissionGranted) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text("通知权限", style = MaterialTheme.typography.titleMedium)
                        Spacer(Modifier.height(8.dp))
                        Text("启动接收服务前需要允许通知。")
                        Spacer(Modifier.height(12.dp))
                        Button(onClick = onRequestNotificationPermission) {
                            Text("允许通知")
                        }
                    }
                }
            }
        }

        item {
            Text("设备账号", style = MaterialTheme.typography.titleMedium)
        }
        if (dashboard.accounts.isEmpty()) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        "还没有账号。请为相机配置 FTP/SFTP 用户名和密码。",
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            items(dashboard.accounts) { account ->
                AccountMenuRow(account = account, onClick = { onOpenAccount(account) })
            }
        }
        item {
            Button(
                onClick = onAddAccount,
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text("新增账号")
            }
        }

        item {
            Text("导入位置", style = MaterialTheme.typography.titleMedium)
        }
        item {
            SettingsMenuRow(
                title = "选择导入文件夹",
                subtitle = selectedInboxLabel ?: "应用私有收件箱",
                trailing = ">",
                onClick = onChooseInboxDirectory,
            )
        }
        item {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    Text("当前原生收件箱", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    Text(dashboard.receiver.outputLabel, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
    }
}

@Composable
private fun AccountDetailScreen(
    account: DeviceAccount?,
    actionError: String?,
    actionInFlight: String?,
    onClearActionError: () -> Unit,
    onBack: () -> Unit,
    onSaveDeviceAccount: (DeviceAccount, String?) -> Unit,
    onDeleteDeviceAccount: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val isNew = account == null
    var deviceName by remember(account?.username) { mutableStateOf(account?.deviceName.orEmpty()) }
    var username by remember(account?.username) { mutableStateOf(account?.username.orEmpty()) }
    var password by remember(account?.username) { mutableStateOf("") }
    val locked = account?.let { it.online || it.activeConnections > 0 } ?: false
    val actionsEnabled = actionInFlight == null
    val cleanUsername = username.trim()
    val passwordOk = account?.passwordConfigured == true || password.isNotBlank()
    val canSave = actionsEnabled && !locked && cleanUsername.isNotBlank() && passwordOk

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            HeaderWithBack(
                title = if (isNew) "新增账号" else "账号详情",
                subtitle = if (locked) "连接中的账号不可编辑或删除" else "用户名、密码、设备名称",
                onBack = onBack,
            )
        }

        actionError?.let { message ->
            item { ActionMessageCard(title = "操作失败", message = message, onClose = onClearActionError) }
        }

        actionInFlight?.let { action ->
            item { ProcessingCard(action) }
        }

        account?.let {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text("连接状态", style = MaterialTheme.typography.titleMedium)
                        Spacer(Modifier.height(8.dp))
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            ElementTag(
                                text = if (it.online) "在线" else "未连接",
                                color = if (it.online) ElementSuccess else ElementInfo,
                            )
                            ElementTag("连接数 ${it.activeConnections}", ElementBlue)
                        }
                        Spacer(Modifier.height(8.dp))
                        Text("最近来源：${formatEndpoint(it)}")
                        it.lastSeenAtMs?.let { value -> Text("最近在线：${formatEpochMillisForDisplay(value)}") }
                        it.lastDisconnectedAtMs?.let { value ->
                            Text("最近断开：${formatEpochMillisForDisplay(value)}")
                        }
                    }
                }
            }
        }

        item {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    OutlinedTextField(
                        value = username,
                        onValueChange = { username = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("用户名") },
                        singleLine = true,
                        enabled = actionsEnabled && !locked && isNew,
                    )
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(
                        value = deviceName,
                        onValueChange = { deviceName = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text("设备名称") },
                        singleLine = true,
                        enabled = actionsEnabled && !locked,
                    )
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(
                        value = password,
                        onValueChange = { password = it },
                        modifier = Modifier.fillMaxWidth(),
                        label = { Text(if (account?.passwordConfigured == true) "新密码（留空不修改）" else "密码") },
                        singleLine = true,
                        visualTransformation = PasswordVisualTransformation(),
                        enabled = actionsEnabled && !locked,
                    )
                    if (locked) {
                        Spacer(Modifier.height(8.dp))
                        Text("请先等待相机断开或停止接收服务，再修改该账号。")
                    }
                    Spacer(Modifier.height(16.dp))
                    Button(
                        onClick = {
                            val nextUsername = cleanUsername
                            onSaveDeviceAccount(
                                DeviceAccount(
                                    username = nextUsername,
                                    deviceName = deviceName.trim().ifBlank { nextUsername },
                                    passwordConfigured = account?.passwordConfigured == true || password.isNotBlank(),
                                    latestIp = account?.latestIp,
                                    latestPort = account?.latestPort,
                                    activeConnections = account?.activeConnections ?: 0,
                                    lastSeenAtMs = account?.lastSeenAtMs,
                                    lastDisconnectedAtMs = account?.lastDisconnectedAtMs,
                                    online = account?.online ?: false,
                                ),
                                password.takeIf { it.isNotBlank() },
                            )
                            password = ""
                        },
                        enabled = canSave,
                        modifier = Modifier.fillMaxWidth(),
                    ) {
                        Text("保存")
                    }
                    if (!isNew) {
                        Spacer(Modifier.height(8.dp))
                        OutlinedButton(
                            onClick = { onDeleteDeviceAccount(cleanUsername) },
                            enabled = actionsEnabled && !locked,
                            modifier = Modifier.fillMaxWidth(),
                            colors = ButtonDefaults.outlinedButtonColors(contentColor = ElementDanger),
                            border = BorderStroke(1.dp, ElementDanger),
                        ) {
                            Text("删除账号")
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun InboxScreen(
    dashboard: DashboardState,
    gridColumnCount: Int,
    onGridColumnCountChange: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    var selectedSource by remember { mutableStateOf<String?>(null) }
    var selectedFilter by remember { mutableStateOf(InboxFilter.All) }
    var selectedPhoto by remember { mutableStateOf<InboxAsset?>(null) }
    var filterExpanded by remember { mutableStateOf(false) }
    val filteredAssets = remember(dashboard.inbox, selectedSource, selectedFilter) {
        dashboard.inbox
            .filter { asset -> selectedSource == null || asset.sourceLabel() == selectedSource }
            .filter { asset -> selectedFilter.matches(asset) }
    }

    selectedPhoto?.let { photo ->
        BackHandler {
            selectedPhoto = null
        }
        PhotoDetailScreen(
            asset = photo,
            onBack = { selectedPhoto = null },
            modifier = modifier,
        )
        return
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .padding(16.dp),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f)) {
                Text("收件箱", style = MaterialTheme.typography.headlineMedium)
                Spacer(Modifier.height(4.dp))
                Text("照片信息流", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            Spacer(Modifier.width(12.dp))
            GridColumnToggle(
                columnCount = gridColumnCount,
                onColumnCountChange = onGridColumnCountChange,
            )
        }
        Spacer(Modifier.height(10.dp))
        FilterToggleRow(
            selectedSource = selectedSource,
            selectedFilter = selectedFilter,
            expanded = filterExpanded,
            onToggle = { filterExpanded = !filterExpanded },
        )
        if (filterExpanded) {
            Spacer(Modifier.height(8.dp))
            SourceFilterBar(
                selectedSource = selectedSource,
                onSourceChange = { selectedSource = it },
                assets = dashboard.inbox,
            )
            Spacer(Modifier.height(8.dp))
            InboxFilterBar(
                selectedFilter = selectedFilter,
                onFilterChange = { selectedFilter = it },
                assets = dashboard.inbox.filter { selectedSource == null || it.sourceLabel() == selectedSource },
            )
        }
        Spacer(Modifier.height(10.dp))
        if (filteredAssets.isEmpty()) {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Text(
                    if (dashboard.inbox.isEmpty()) "还没有导入文件。" else "当前筛选下没有文件。",
                    modifier = Modifier.padding(16.dp),
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        } else {
            LazyVerticalGrid(
                columns = GridCells.Fixed(gridColumnCount),
                modifier = Modifier.fillMaxSize(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp),
                contentPadding = PaddingValues(bottom = 8.dp),
            ) {
                items(
                    count = filteredAssets.size,
                    key = { index -> filteredAssets[index].id.ifBlank { filteredAssets[index].displayPath } },
                ) { index ->
                    val asset = filteredAssets[index]
                    CompactPhotoTile(asset = asset, onClick = { selectedPhoto = asset })
                }
            }
        }
    }
}

@Composable
private fun GridColumnToggle(
    columnCount: Int,
    onColumnCountChange: (Int) -> Unit,
) {
    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        listOf(2 to "2列", 3 to "3列").forEach { (count, label) ->
            OutlinedButton(
                onClick = { onColumnCountChange(count) },
                modifier = Modifier
                    .height(30.dp)
                    .defaultMinSize(minWidth = 1.dp, minHeight = 1.dp),
                border = BorderStroke(1.dp, if (columnCount == count) ElementBlue else ElementBorder),
                colors = ButtonDefaults.outlinedButtonColors(
                    containerColor = if (columnCount == count) ElementBlue else Color.White,
                    contentColor = if (columnCount == count) Color.White else MaterialTheme.colorScheme.onSurfaceVariant,
                ),
                shape = elementShape,
                contentPadding = PaddingValues(horizontal = 9.dp, vertical = 0.dp),
            ) {
                Text(label, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            }
        }
    }
}

@Composable
private fun FilterToggleRow(
    selectedSource: String?,
    selectedFilter: InboxFilter,
    expanded: Boolean,
    onToggle: () -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onToggle)
            .padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column {
            Text("筛选", style = MaterialTheme.typography.titleSmall)
            Spacer(Modifier.height(2.dp))
            Text(
                listOf(selectedSource ?: "全部来源", selectedFilter.label).joinToString(" / "),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.bodySmall,
            )
        }
        Text(if (expanded) "收起 ▲" else "展开 ▼", color = ElementBlue, fontWeight = FontWeight.SemiBold)
    }
}

@Composable
private fun SourceFilterBar(
    selectedSource: String?,
    onSourceChange: (String?) -> Unit,
    assets: List<InboxAsset>,
) {
    val sources = remember(assets) {
        assets.map { it.sourceLabel() }.distinct().sorted()
    }
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        item {
            FilterChipButton(
                label = "全部来源 ${assets.size}",
                selected = selectedSource == null,
                onClick = { onSourceChange(null) },
            )
        }
        items(sources) { source ->
            val count = assets.count { it.sourceLabel() == source }
            FilterChipButton(
                label = "$source $count",
                selected = selectedSource == source,
                onClick = { onSourceChange(source) },
            )
        }
    }
}

@Composable
private fun InboxFilterBar(
    selectedFilter: InboxFilter,
    onFilterChange: (InboxFilter) -> Unit,
    assets: List<InboxAsset>,
) {
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(InboxFilter.entries) { filter ->
            val count = assets.count { filter.matches(it) }
            FilterChipButton(
                label = "${filter.label} $count",
                selected = selectedFilter == filter,
                onClick = { onFilterChange(filter) },
            )
        }
    }
}

@Composable
private fun FilterChipButton(
    label: String,
    selected: Boolean,
    onClick: () -> Unit,
) {
    OutlinedButton(
        onClick = onClick,
        modifier = Modifier
            .height(30.dp)
            .defaultMinSize(minWidth = 1.dp, minHeight = 1.dp),
        border = BorderStroke(1.dp, if (selected) ElementBlue else ElementBorder),
        colors = ButtonDefaults.outlinedButtonColors(
            containerColor = if (selected) ElementBlue else Color.White,
            contentColor = if (selected) Color.White else MaterialTheme.colorScheme.onSurface,
        ),
        shape = elementShape,
        contentPadding = PaddingValues(horizontal = 10.dp, vertical = 4.dp),
    ) {
        Text(label, style = MaterialTheme.typography.labelMedium)
    }
}

@Composable
private fun CompactPhotoTile(asset: InboxAsset, onClick: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick),
    ) {
        PhotoPreview(
            asset = asset,
            compactFallback = true,
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(1f),
        )
        Spacer(Modifier.height(5.dp))
        Text(
            asset.filename(),
            fontSize = 12.sp,
            lineHeight = 14.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Spacer(Modifier.height(2.dp))
        Text(
            listOf(asset.sourceLabel(), asset.formatBadges()).joinToString(" · "),
            fontSize = 10.sp,
            lineHeight = 12.sp,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun PhotoDetailScreen(
    asset: InboxAsset,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            HeaderWithBack(
                title = asset.groupTitle(),
                subtitle = "照片详情",
                onBack = onBack,
            )
        }
        item {
            PhotoPreview(
                asset = asset,
                modifier = Modifier
                    .fillMaxWidth()
                    .aspectRatio(1f),
            )
        }
        item {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    Text("来源信息", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    DetailLine("来源", asset.sourceLabel())
                    DetailLine("账号", asset.username ?: "未记录")
                    DetailLine("原始路径", asset.originalPath ?: asset.displayPath)
                    DetailLine("接收时间", formatEpochMillisTextForDisplay(asset.receivedAt))
                    DetailLine("文件大小", asset.sizeBytes?.let { "$it bytes" } ?: "未记录")
                }
            }
        }
        item {
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(16.dp)) {
                    Text("文件组", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(8.dp))
                    DetailLine("主文件", asset.displayPath)
                    DetailLine("RAW", asset.rawPath ?: "无")
                    DetailLine("JPEG", asset.jpegPath ?: "无")
                    DetailLine("视频", asset.videoPath ?: "无")
                }
            }
        }
    }
}

@Composable
private fun PhotoPreview(
    asset: InboxAsset,
    modifier: Modifier = Modifier,
    compactFallback: Boolean = false,
) {
    val context = LocalContext.current
    val previewLocation = asset.previewLocation
    val bitmap = remember(previewLocation) {
        loadPreviewBitmap(context, previewLocation)
    }

    Box(
        modifier = modifier
            .clip(elementShape)
            .background(Color(0xFFE4E7ED)),
        contentAlignment = Alignment.Center,
    ) {
        if (bitmap != null) {
            Image(
                bitmap = bitmap.asImageBitmap(),
                contentDescription = asset.groupTitle(),
                modifier = Modifier.fillMaxSize(),
                contentScale = ContentScale.Crop,
            )
        } else {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                if (compactFallback) {
                    Text(
                        asset.formatBadges(),
                        color = ElementInfo,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                } else {
                    Text(
                        if (asset.rawPath != null) "RAW 预览待生成" else "暂无预览",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        fontWeight = FontWeight.SemiBold,
                    )
                    Spacer(Modifier.height(4.dp))
                    Text(asset.formatBadges(), color = ElementInfo)
                }
            }
        }
    }
}

private fun loadPreviewBitmap(context: Context, location: String?): Bitmap? {
    if (location.isNullOrBlank()) {
        return null
    }
    return runCatching {
        if (location.startsWith("content://")) {
            context.contentResolver.openInputStream(Uri.parse(location))?.use(BitmapFactory::decodeStream)
        } else {
            File(location).inputStream().use(BitmapFactory::decodeStream)
        }
    }.getOrNull()
}

@Composable
private fun DetailLine(label: String, value: String) {
    Column(Modifier.padding(vertical = 4.dp)) {
        Text(label, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text(value, style = MaterialTheme.typography.bodyLarge)
    }
}

@Composable
private fun TransfersScreen(
    dashboard: DashboardState,
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Text("传输记录", style = MaterialTheme.typography.headlineMedium)
        }
        if (dashboard.transfers.isEmpty()) {
            item { Text("还没有传输记录。") }
        } else {
            items(dashboard.transfers) { transfer ->
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(16.dp)) {
                        Text(transfer.displayPath, style = MaterialTheme.typography.titleMedium)
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            ElementTag(
                                text = transferStatusLabel(transfer.status),
                                color = if (transfer.status == "Failed") ElementDanger else ElementSuccess,
                            )
                            Text(transfer.id, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        transfer.message?.let { Text(it) }
                    }
                }
            }
        }
    }
}

@Composable
private fun HeaderWithBack(
    title: String,
    subtitle: String,
    onBack: () -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconButton(onClick = onBack) {
            Icon(Icons.AutoMirrored.Outlined.ArrowBack, contentDescription = "返回")
        }
        Column {
            Text(title, style = MaterialTheme.typography.headlineMedium)
            Spacer(Modifier.height(4.dp))
            Text(subtitle, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
private fun ActionMessageCard(
    title: String,
    message: String,
    onClose: () -> Unit,
) {
    ElementCard(modifier = Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp)) {
            Text(title, style = MaterialTheme.typography.titleMedium)
            Spacer(Modifier.height(8.dp))
            Text(message)
            Spacer(Modifier.height(12.dp))
            Button(onClick = onClose) {
                Text("关闭")
            }
        }
    }
}

@Composable
private fun ProcessingCard(action: String) {
    ElementCard(modifier = Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp)) {
            Text("处理中", style = MaterialTheme.typography.titleMedium)
            Spacer(Modifier.height(8.dp))
            Text(action)
        }
    }
}

@Composable
private fun AccountMenuRow(account: DeviceAccount, onClick: () -> Unit) {
    SettingsMenuRow(
        title = account.deviceName.ifBlank { account.username },
        subtitle = "${account.username} · ${formatEndpoint(account)}",
        trailing = if (account.online || account.activeConnections > 0) "在线 >" else "未连接 >",
        onClick = onClick,
        trailingColor = if (account.online || account.activeConnections > 0) ElementSuccess else ElementInfo,
    )
}

@Composable
private fun SettingsMenuRow(
    title: String,
    subtitle: String,
    trailing: String,
    onClick: () -> Unit,
    trailingColor: Color = MaterialTheme.colorScheme.onSurfaceVariant,
) {
    ElementCard(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick),
    ) {
        Row(
            modifier = Modifier.padding(16.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f)) {
                Text(title, style = MaterialTheme.typography.titleMedium)
                Spacer(Modifier.height(4.dp))
                Text(
                    subtitle,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Spacer(Modifier.width(12.dp))
            Text(trailing, color = trailingColor, fontWeight = FontWeight.SemiBold)
        }
    }
}

private fun receiverPhaseLabel(value: String): String = when (value) {
    "Running" -> "运行中"
    "Stopped" -> "已停止"
    "Unknown" -> "未知"
    else -> value
}

private fun authModeLabel(value: String): String = when (value) {
    "Accounts" -> "账号认证"
    "Open" -> "开放"
    "Unknown" -> "未知"
    else -> value
}

private fun transferStatusLabel(value: String): String = when (value) {
    "Completed" -> "已完成"
    "Failed" -> "失败"
    "Pending" -> "等待中"
    else -> value
}

private fun normalizeListenHost(value: String?): String {
    val trimmed = value?.trim().orEmpty()
    return if (trimmed.isBlank() || trimmed.equals("null", ignoreCase = true)) {
        DEFAULT_LISTEN_HOST
    } else {
        trimmed
    }
}

private fun formatEndpoint(account: DeviceAccount): String {
    val host = account.latestIp?.takeIf { it.isNotBlank() } ?: "暂无来源"
    val port = account.latestPort?.let { ":$it" } ?: ""
    return "$host$port"
}

private fun InboxFilter.matches(asset: InboxAsset): Boolean = when (this) {
    InboxFilter.All -> true
    InboxFilter.Raw -> asset.rawPath != null || asset.format.isRawFormat()
    InboxFilter.Jpeg -> asset.jpegPath != null || asset.format.lowercase() in setOf("jpeg", "jpg")
    InboxFilter.Video -> asset.videoPath != null || asset.format.lowercase() in setOf("mov", "mp4")
}

private fun InboxAsset.filename(): String =
    displayPath.substringAfterLast('/').substringAfterLast('\\').ifBlank { displayPath }

private fun InboxAsset.groupTitle(): String =
    groupKey.ifBlank { filename().substringBeforeLast('.', filename()) }

private fun InboxAsset.sourceLabel(): String =
    displaySource?.takeIf { it.isNotBlank() }
        ?: username?.takeIf { it.isNotBlank() }?.let { "账号：$it" }
        ?: sourceGroupLabel(displayPath)

private fun InboxAsset.formatBadges(): String =
    buildList {
        if (jpegPath != null || format.lowercase() in setOf("jpeg", "jpg")) add("JPG")
        if (rawPath != null || format.isRawFormat()) add("RAW")
        if (videoPath != null || format.lowercase() in setOf("mov", "mp4")) add("视频")
        if (isEmpty()) add(format.ifBlank { "未知" })
    }.joinToString(" · ")

private fun String.isRawFormat(): Boolean =
    lowercase() in setOf("raw", "nef", "nrw", "cr3", "cr2", "arw", "raf", "orf", "rw2", "pef", "dng")

private fun formatColor(format: String): Color = when (format.lowercase()) {
    "raw", "nef", "cr3", "cr2", "arw", "raf", "orf", "rw2", "dng" -> ElementWarning
    "jpeg", "jpg" -> ElementSuccess
    "mov", "mp4" -> ElementBlue
    else -> ElementInfo
}

private fun sourceGroupLabel(displayPath: String): String =
    displayPath.substringBeforeLast('/', missingDelimiterValue = "未分组").ifBlank { "未分组" }
