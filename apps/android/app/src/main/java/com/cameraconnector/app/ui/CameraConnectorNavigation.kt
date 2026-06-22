package com.cameraconnector.app.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp

@Composable
internal fun CameraConnectorBottomBar(
    selected: GlobalDestination,
    onSelect: (GlobalDestination) -> Unit,
) {
    Box(
        modifier = Modifier.fillMaxWidth(),
        contentAlignment = Alignment.BottomCenter,
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(78.dp),
        ) {
            Canvas(modifier = Modifier.fillMaxSize()) {
                val centerX = size.width / 2f
                val top = 10.dp.toPx()
                val dip = 20.dp.toPx()
                val shoulder = 66.dp.toPx()
                val neck = 38.dp.toPx()
                val dock = Path().apply {
                    moveTo(0f, top)
                    lineTo(centerX - shoulder, top)
                    cubicTo(
                        centerX - 54.dp.toPx(),
                        top,
                        centerX - 50.dp.toPx(),
                        top + dip,
                        centerX - neck,
                        top + dip,
                    )
                    lineTo(centerX + neck, top + dip)
                    cubicTo(
                        centerX + 50.dp.toPx(),
                        top + dip,
                        centerX + 54.dp.toPx(),
                        top,
                        centerX + shoulder,
                        top,
                    )
                    lineTo(size.width, top)
                    lineTo(size.width, size.height)
                    lineTo(0f, size.height)
                    close()
                }
                drawPath(
                    path = dock,
                    color = ElementSurface,
                )
                drawPath(
                    path = dock,
                    color = ElementCardBorder.copy(alpha = 0.72f),
                    style = Stroke(width = 1.dp.toPx(), cap = StrokeCap.Round),
                )
            }
            SecondaryBottomDestination(
                destination = GlobalDestination.Settings,
                selected = selected == GlobalDestination.Settings,
                onClick = { onSelect(GlobalDestination.Settings) },
                modifier = Modifier
                    .align(Alignment.TopStart)
                    .padding(start = 42.dp, top = 24.dp),
            )
            PrimaryBottomDestination(
                destination = GlobalDestination.Projects,
                selected = selected == GlobalDestination.Projects,
                onClick = { onSelect(GlobalDestination.Projects) },
                modifier = Modifier
                    .align(Alignment.TopCenter)
                    .offset(y = 2.dp),
            )
            SecondaryBottomDestination(
                destination = GlobalDestination.Accounts,
                selected = selected == GlobalDestination.Accounts,
                onClick = { onSelect(GlobalDestination.Accounts) },
                modifier = Modifier
                    .align(Alignment.TopEnd)
                    .padding(end = 42.dp, top = 24.dp),
            )
        }
    }
}

@Composable
private fun PrimaryBottomDestination(
    destination: GlobalDestination,
    selected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier
            .size(52.dp)
            .clip(CircleShape)
            .clickable(onClick = onClick),
        color = if (selected) ElementBlue else ElementControlSurface,
        contentColor = if (selected) ElementOnAccent else MaterialTheme.colorScheme.onSurfaceVariant,
        shape = CircleShape,
        border = BorderStroke(1.dp, if (selected) ElementBlue else ElementCardBorder),
        shadowElevation = if (selected) 10.dp else 2.dp,
    ) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = destination.icon(),
                contentDescription = destination.label,
                modifier = Modifier.size(24.dp),
            )
        }
    }
}

@Composable
private fun SecondaryBottomDestination(
    destination: GlobalDestination,
    selected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier
            .size(40.dp)
            .clip(CircleShape)
            .clickable(onClick = onClick),
        color = if (selected) ElementBlueSoft else Color.Transparent,
        contentColor = if (selected) ElementBlue else MaterialTheme.colorScheme.onSurfaceVariant,
        shape = CircleShape,
        border = BorderStroke(
            1.dp,
            if (selected) ElementBlue.copy(alpha = 0.52f) else Color.Transparent,
        ),
    ) {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = destination.icon(),
                contentDescription = destination.label,
                modifier = Modifier.size(21.dp),
            )
        }
    }
}

private fun GlobalDestination.icon(): ImageVector = when (this) {
    GlobalDestination.Projects -> Icons.Outlined.Home
    GlobalDestination.Accounts -> Icons.Outlined.Person
    GlobalDestination.Settings -> Icons.Outlined.Settings
}
