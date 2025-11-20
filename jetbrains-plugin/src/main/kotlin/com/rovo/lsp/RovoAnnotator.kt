package com.rovo.lsp

import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.Annotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.util.PsiTreeUtil

/**
 * Provides custom syntax highlighting for Rovo annotations in doc comments.
 *
 * This annotator is context-aware and only highlights annotations near #[rovo] attributes,
 * similar to the Neovim plugin behavior.
 */
class RovoAnnotator : Annotator {

    companion object {
        // Rovo annotation keywords
        private val ANNOTATIONS = setOf("@response", "@tag", "@security", "@example", "@id", "@hidden")

        // Security scheme names
        private val SECURITY_SCHEMES = setOf("bearer", "basic", "apiKey", "oauth2")

        // Regex patterns for matching
        private val ROVO_ATTR_PATTERN = Regex("""#\[\s*\w*[::\w]*rovo\s*]""")
        private val DOC_COMMENT_PATTERN = Regex("""^\s*///(.*)""")
        private val ANNOTATION_PATTERN = Regex("""(@\w+)""")
        private val STATUS_CODE_PATTERN = Regex("""@\w+\s+(\d{3})""")
        private val SECURITY_SCHEME_PATTERN = Regex("""@security\s+(\w+)""")
    }

    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        // Only process Rust files
        val file = element.containingFile ?: return
        if (!file.name.endsWith(".rs")) return

        // Check if this file contains any #[rovo] attributes
        if (!hasRovoAttributes(file)) return

        // Only process elements that are in doc comments near #[rovo]
        val text = element.text
        if (!isDocComment(text)) return

        // Check if this doc comment is near a #[rovo] attribute
        if (!isNearRovoAttribute(element)) return

        // Extract the comment content (without ///)
        val commentContent = extractCommentContent(text) ?: return

        // Highlight annotations (@response, @tag, etc.)
        highlightAnnotations(commentContent, element, holder)

        // Highlight status codes (200, 404, etc.)
        highlightStatusCodes(commentContent, element, holder)

        // Highlight security schemes (bearer, basic, etc.)
        highlightSecuritySchemes(commentContent, element, holder)
    }

    private fun hasRovoAttributes(file: PsiFile): Boolean {
        val fileText = file.text
        return ROVO_ATTR_PATTERN.containsMatchIn(fileText)
    }

    private fun isDocComment(text: String): Boolean {
        return text.trimStart().startsWith("///")
    }

    private fun isNearRovoAttribute(element: PsiElement): Boolean {
        // Look for #[rovo] attribute within the next 10 siblings
        var current: PsiElement? = element.nextSibling
        var distance = 0

        while (current != null && distance < 30) {
            val text = current.text
            if (ROVO_ATTR_PATTERN.containsMatchIn(text)) {
                return true
            }
            // Stop if we hit a non-comment, non-whitespace element that's not an attribute
            if (!text.matches(Regex("""^\s*$""")) &&
                !text.trimStart().startsWith("///") &&
                !text.trimStart().startsWith("#[")) {
                break
            }
            current = current.nextSibling
            distance++
        }

        return false
    }

    private fun extractCommentContent(text: String): String? {
        val match = DOC_COMMENT_PATTERN.find(text) ?: return null
        return match.groupValues[1]
    }

    private fun highlightAnnotations(content: String, element: PsiElement, holder: AnnotationHolder) {
        ANNOTATION_PATTERN.findAll(content).forEach { match ->
            val annotation = match.value
            if (annotation in ANNOTATIONS) {
                val startOffset = element.textRange.startOffset + element.text.indexOf(annotation)
                val endOffset = startOffset + annotation.length

                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                    .range(TextRange(startOffset, endOffset))
                    .textAttributes(DefaultLanguageHighlighterColors.KEYWORD)
                    .create()
            }
        }
    }

    private fun highlightStatusCodes(content: String, element: PsiElement, holder: AnnotationHolder) {
        STATUS_CODE_PATTERN.findAll(content).forEach { match ->
            val statusCode = match.groupValues[1]
            val code = statusCode.toIntOrNull() ?: return@forEach

            // Only highlight valid HTTP status codes (100-599)
            if (code in 100..599) {
                val startOffset = element.textRange.startOffset +
                    element.text.indexOf(statusCode, element.text.indexOf("@"))
                val endOffset = startOffset + statusCode.length

                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                    .range(TextRange(startOffset, endOffset))
                    .textAttributes(DefaultLanguageHighlighterColors.NUMBER)
                    .create()
            }
        }
    }

    private fun highlightSecuritySchemes(content: String, element: PsiElement, holder: AnnotationHolder) {
        SECURITY_SCHEME_PATTERN.findAll(content).forEach { match ->
            val scheme = match.groupValues[1]

            if (scheme in SECURITY_SCHEMES) {
                val startOffset = element.textRange.startOffset +
                    element.text.indexOf(scheme, element.text.indexOf("@security"))
                val endOffset = startOffset + scheme.length

                holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
                    .range(TextRange(startOffset, endOffset))
                    .textAttributes(DefaultLanguageHighlighterColors.STRING)
                    .create()
            }
        }
    }
}
