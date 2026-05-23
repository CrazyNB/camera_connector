package com.cameraconnector.app.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
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
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
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
                host = "0.0.0.0",
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
            ElementCard(modifier = Modifier.fillMaxWidth()) {
                Column(
                    modifier = Modifier.padding(16.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Text(
                        if (dashboard.receiver.running) "接收服务运行中" else "接收服务已停止",
                        style = MaterialTheme.typography.titleMedium,
                    )
                    Spacer(Modifier.height(8.dp))
                    Text(
                        "${dashboard.receiver.protocol} $displayHost:${dashboard.receiver.port}",
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    Spacer(Modifier.height(20.dp))
                    PowerButton(
                        running = dashboard.receiver.running,
                        enabled = actionsEnabled && (dashboard.receiver.running || notificationPermissionGranted),
                        onClick = onToggleReceiver,
                    )
                    Spacer(Modifier.height(20.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        ElementTag(
                            text = receiverPhaseLabel(dashboard.receiver.phase),
                            color = if (dashboard.receiver.running) ElementSuccess else ElementInfo,
                        )
                        ElementTag(
                            text = if (onlineConnections > 0) "在线连接 $onlineConnections" else "未连接",
                            color = if (onlineConnections > 0) ElementSuccess else ElementInfo,
                        )
                        ElementTag(text = "已配置账号 ${dashboard.accounts.size}", color = ElementBlue)
                    }
                    dashboard.receiver.message?.let {
                        Spacer(Modifier.height(8.dp))
                        Text(it, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
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
    modifier: Modifier = Modifier,
) {
    var selectedFilter by remember { mutableStateOf(InboxFilter.All) }
    val filteredAssets = remember(dashboard.inbox, selectedFilter) {
        dashboard.inbox.filter { asset -> selectedFilter.matches(asset) }
    }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Column {
                Text("收件箱", style = MaterialTheme.typography.headlineMedium)
                Spacer(Modifier.height(4.dp))
                Text("按照片信息流查看，可用标签快速筛选。", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
        item {
            InboxFilterBar(
                selectedFilter = selectedFilter,
                onFilterChange = { selectedFilter = it },
                assets = dashboard.inbox,
            )
        }
        if (filteredAssets.isEmpty()) {
            item {
                ElementCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        if (dashboard.inbox.isEmpty()) "还没有导入文件。" else "当前筛选下没有文件。",
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            items(filteredAssets) { asset ->
                PhotoInfoCard(asset = asset)
            }
        }
    }
}

@Composable
private fun InboxFilterBar(
    selectedFilter: InboxFilter,
    onFilterChange: (InboxFilter) -> Unit,
    assets: List<InboxAsset>,
) {
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        InboxFilter.entries.forEach { filter ->
            val count = assets.count { filter.matches(it) }
            ProtocolSegment(
                label = "${filter.label} $count",
                selected = selectedFilter == filter,
                enabled = true,
                onClick = { onFilterChange(filter) },
                modifier = Modifier.weight(1f),
            )
        }
    }
}

@Composable
private fun PhotoInfoCard(asset: InboxAsset) {
    ElementCard(modifier = Modifier.fillMaxWidth()) {
        Column(Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.Top,
            ) {
                Column(Modifier.weight(1f)) {
                    Text(
                        asset.filename(),
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Spacer(Modifier.height(4.dp))
                    Text(
                        asset.displayPath,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                Spacer(Modifier.width(8.dp))
                ElementTag(asset.format.ifBlank { "未知" }, formatColor(asset.format))
            }
            Spacer(Modifier.height(12.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                ElementTag("接收 ${formatEpochMillisTextForDisplay(asset.receivedAt)}", ElementInfo)
                ElementTag(sourceGroupLabel(asset.displayPath), ElementBlue)
            }
        }
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
        "0.0.0.0"
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
    InboxFilter.Raw -> asset.format.lowercase() in setOf("raw", "nef", "cr3", "cr2", "arw", "raf", "orf", "rw2", "dng")
    InboxFilter.Jpeg -> asset.format.lowercase() in setOf("jpeg", "jpg")
    InboxFilter.Video -> asset.format.lowercase() in setOf("mov", "mp4")
}

private fun InboxAsset.filename(): String =
    displayPath.substringAfterLast('/').substringAfterLast('\\').ifBlank { displayPath }

private fun formatColor(format: String): Color = when (format.lowercase()) {
    "raw", "nef", "cr3", "cr2", "arw", "raf", "orf", "rw2", "dng" -> ElementWarning
    "jpeg", "jpg" -> ElementSuccess
    "mov", "mp4" -> ElementBlue
    else -> ElementInfo
}

private fun sourceGroupLabel(displayPath: String): String =
    displayPath.substringBeforeLast('/', missingDelimiterValue = "未分组").ifBlank { "未分组" }
