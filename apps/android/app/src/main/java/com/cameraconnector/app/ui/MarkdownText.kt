package com.cameraconnector.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

private sealed interface MarkdownBlock {
    data class Paragraph(val text: String) : MarkdownBlock
    data class Heading(val level: Int, val text: String) : MarkdownBlock
    data class ListItem(val text: String) : MarkdownBlock
    data class Quote(val text: String) : MarkdownBlock
    data class Code(val text: String) : MarkdownBlock
}

@Composable
internal fun MarkdownText(
    markdown: String,
    modifier: Modifier = Modifier,
    color: Color = MaterialTheme.colorScheme.onSurface,
    compact: Boolean = false,
    maxLines: Int = Int.MAX_VALUE,
) {
    val blocks = remember(markdown) { parseMarkdownBlocks(markdown) }
    val textStyle = MaterialTheme.typography.bodyMedium
    val secondaryColor = MaterialTheme.colorScheme.onSurfaceVariant
    val codeBackground = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.45f)

    Column(
        modifier = modifier,
        verticalArrangement = Arrangement.spacedBy(if (compact) 4.dp else 8.dp),
    ) {
        blocks.forEach { block ->
            when (block) {
                is MarkdownBlock.Heading -> Text(
                    inlineMarkdown(
                        block.text,
                        color = color,
                        codeBackground = codeBackground,
                    ),
                    style = headingStyle(block.level),
                    maxLines = maxLines,
                    overflow = TextOverflow.Ellipsis,
                )

                is MarkdownBlock.ListItem -> Row {
                    Text("•", color = ElementBlue, fontWeight = FontWeight.SemiBold)
                    Spacer(Modifier.width(8.dp))
                    Text(
                        inlineMarkdown(block.text, color = color, codeBackground = codeBackground),
                        style = textStyle,
                        color = color,
                        maxLines = maxLines,
                        overflow = TextOverflow.Ellipsis,
                    )
                }

                is MarkdownBlock.Quote -> Text(
                    inlineMarkdown(block.text, color = secondaryColor, codeBackground = codeBackground),
                    modifier = Modifier
                        .clip(RoundedCornerShape(6.dp))
                        .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.22f)),
                    style = textStyle,
                    color = secondaryColor,
                    maxLines = maxLines,
                    overflow = TextOverflow.Ellipsis,
                )

                is MarkdownBlock.Code -> Text(
                    block.text,
                    modifier = Modifier
                        .clip(RoundedCornerShape(6.dp))
                        .background(codeBackground),
                    style = textStyle.copy(fontFamily = FontFamily.Monospace),
                    color = color,
                    maxLines = maxLines,
                    overflow = TextOverflow.Ellipsis,
                )

                is MarkdownBlock.Paragraph -> Text(
                    inlineMarkdown(block.text, color = color, codeBackground = codeBackground),
                    style = textStyle,
                    color = color,
                    maxLines = maxLines,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

private fun parseMarkdownBlocks(markdown: String): List<MarkdownBlock> {
    val blocks = mutableListOf<MarkdownBlock>()
    val paragraph = StringBuilder()
    val code = StringBuilder()
    var inCode = false

    fun flushParagraph() {
        val text = paragraph.toString().trim()
        if (text.isNotEmpty()) {
            blocks += MarkdownBlock.Paragraph(text)
        }
        paragraph.clear()
    }

    fun flushCode() {
        blocks += MarkdownBlock.Code(code.toString().trimEnd())
        code.clear()
    }

    markdown.lines().forEach { rawLine ->
        val line = rawLine.trimEnd()
        val trimmed = line.trim()
        if (trimmed.startsWith("```")) {
            if (inCode) {
                flushCode()
            } else {
                flushParagraph()
            }
            inCode = !inCode
            return@forEach
        }
        if (inCode) {
            code.appendLine(rawLine)
            return@forEach
        }
        if (trimmed.isBlank()) {
            flushParagraph()
            return@forEach
        }

        parseStandaloneMarkdownLine(trimmed)?.let { block ->
            flushParagraph()
            blocks += block
            return@forEach
        }

        if (paragraph.isNotEmpty()) {
            paragraph.append('\n')
        }
        paragraph.append(trimmed)
    }

    if (inCode) {
        flushCode()
    }
    flushParagraph()
    return blocks.ifEmpty { listOf(MarkdownBlock.Paragraph("")) }
}

private fun parseStandaloneMarkdownLine(trimmed: String): MarkdownBlock? {
    val headingLevel = trimmed.takeWhile { it == '#' }.length
    if (headingLevel in 1..3 && trimmed.getOrNull(headingLevel) == ' ') {
        return MarkdownBlock.Heading(headingLevel, trimmed.drop(headingLevel).trim())
    }
    if (trimmed.startsWith(">")) {
        return MarkdownBlock.Quote(trimmed.drop(1).trim())
    }
    if (trimmed.startsWith("- ") || trimmed.startsWith("* ")) {
        return MarkdownBlock.ListItem(trimmed.drop(2).trim())
    }
    val numberedList = Regex("""^\d+[.)]\s+(.+)$""").matchEntire(trimmed)
    if (numberedList != null) {
        return MarkdownBlock.ListItem(numberedList.groupValues[1].trim())
    }
    return null
}

@Composable
private fun headingStyle(level: Int): TextStyle =
    when (level) {
        1 -> MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight.SemiBold)
        2 -> MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.SemiBold)
        else -> MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold)
    }

private fun inlineMarkdown(
    value: String,
    color: Color,
    codeBackground: Color,
): AnnotatedString =
    buildAnnotatedString {
        appendInlineMarkdown(value, color, codeBackground)
    }

private fun AnnotatedString.Builder.appendInlineMarkdown(
    value: String,
    color: Color,
    codeBackground: Color,
) {
    var index = 0
    while (index < value.length) {
        val codeStart = value.indexOf('`', index)
        val boldStart = value.indexOf("**", index)
        val italicStart = value.indexOf('*', index)
        val next = listOf(codeStart, boldStart, italicStart)
            .filter { it >= index }
            .minOrNull()

        if (next == null) {
            append(value.substring(index))
            return
        }
        if (next > index) {
            append(value.substring(index, next))
        }

        when {
            next == codeStart -> {
                val end = value.indexOf('`', next + 1)
                if (end == -1) {
                    append(value.substring(next))
                    return
                }
                val content = value.substring(next + 1, end)
                pushStyle(
                    SpanStyle(
                        color = color,
                        background = codeBackground,
                        fontFamily = FontFamily.Monospace,
                    ),
                )
                append(content)
                pop()
                index = end + 1
            }

            next == boldStart -> {
                val end = value.indexOf("**", next + 2)
                if (end == -1) {
                    append(value.substring(next))
                    return
                }
                pushStyle(SpanStyle(fontWeight = FontWeight.SemiBold))
                append(value.substring(next + 2, end))
                pop()
                index = end + 2
            }

            else -> {
                val end = value.indexOf('*', next + 1)
                if (end == -1) {
                    append(value.substring(next))
                    return
                }
                pushStyle(SpanStyle(fontStyle = FontStyle.Italic))
                append(value.substring(next + 1, end))
                pop()
                index = end + 1
            }
        }
    }
}
