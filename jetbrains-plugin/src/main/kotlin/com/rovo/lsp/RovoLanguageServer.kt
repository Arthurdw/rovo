package com.rovo.lsp

import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.redhat.devtools.lsp4ij.LanguageServerFactory
import com.redhat.devtools.lsp4ij.client.LanguageClientImpl
import com.redhat.devtools.lsp4ij.server.StreamConnectionProvider
import org.eclipse.lsp4j.services.LanguageServer
import java.io.InputStream
import java.io.OutputStream

class RovoLanguageServerFactory : LanguageServerFactory {
    override fun createConnectionProvider(project: Project): StreamConnectionProvider {
        return RovoStreamConnectionProvider(project)
    }

    override fun createLanguageClient(project: Project): LanguageClientImpl {
        return LanguageClientImpl(project)
    }
}

class RovoStreamConnectionProvider(private val project: Project) : StreamConnectionProvider {
    private var process: Process? = null

    companion object {
        private val LOG = Logger.getInstance(RovoStreamConnectionProvider::class.java)
    }

    override fun start() {
        try {
            val command = findRovoLsp()
            if (command == null) {
                val msg = "rovo-lsp not found. Please install it: cargo install rovo-lsp"
                LOG.error(msg)
                showNotification("Rovo LSP Error", msg, NotificationType.ERROR)
                throw RuntimeException(msg)
            }

            LOG.info("Starting rovo-lsp from: $command")
            process = ProcessBuilder(command)
                .start()

            if (process == null || !process!!.isAlive) {
                val msg = "Failed to start rovo-lsp process"
                LOG.error(msg)
                showNotification("Rovo LSP Error", msg, NotificationType.ERROR)
                throw RuntimeException(msg)
            }

            LOG.info("rovo-lsp started successfully with pid: ${process?.pid()}")
        } catch (e: Exception) {
            LOG.error("Error starting rovo-lsp", e)
            showNotification("Rovo LSP Error", "Error starting LSP server: ${e.message}", NotificationType.ERROR)
            throw e
        }
    }

    private fun showNotification(title: String, content: String, type: NotificationType) {
        NotificationGroupManager.getInstance()
            .getNotificationGroup("Rovo")
            .createNotification(title, content, type)
            .notify(project)
    }

    override fun getInputStream(): InputStream? {
        return process?.inputStream
    }

    override fun getOutputStream(): OutputStream? {
        return process?.outputStream
    }

    override fun stop() {
        process?.destroy()
        process = null
    }

    private fun findRovoLsp(): String? {
        // Try common installation locations
        val commonPaths = listOf(
            System.getProperty("user.home") + "/.cargo/bin/rovo-lsp",
            "/usr/local/bin/rovo-lsp",
            "/usr/bin/rovo-lsp",
            "rovo-lsp" // Try PATH as fallback
        )

        for (path in commonPaths) {
            val file = java.io.File(path)
            if (path != "rovo-lsp" && file.exists() && file.canExecute()) {
                LOG.info("Found rovo-lsp at: $path")
                return path
            }
        }

        // Try using 'which' with full shell environment
        try {
            val proc = ProcessBuilder("bash", "-c", "which rovo-lsp")
                .redirectOutput(ProcessBuilder.Redirect.PIPE)
                .start()
            proc.waitFor()
            if (proc.exitValue() == 0) {
                val path = proc.inputStream.bufferedReader().readText().trim()
                if (path.isNotEmpty()) {
                    LOG.info("Found rovo-lsp via which: $path")
                    return path
                }
            }
        } catch (e: Exception) {
            LOG.warn("Failed to run 'which' command", e)
        }

        // Last resort: try just "rovo-lsp" and hope it's in PATH
        LOG.warn("Could not find rovo-lsp in common locations, trying PATH")
        return "rovo-lsp"
    }
}
