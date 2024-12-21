package io.codehistorian.eclipse.service;

import com.google.gson.Gson;
import org.eclipse.core.resources.IProject;
import org.eclipse.core.runtime.IProgressMonitor;
import org.eclipse.core.runtime.IStatus;
import org.eclipse.core.runtime.Status;
import org.eclipse.core.runtime.jobs.Job;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.util.concurrent.CompletableFuture;

public class HistorianService {
    private static final String API_URL = "http://localhost:8080/api";
    private final HttpClient client;
    private final Gson gson;

    public HistorianService() {
        this.client = HttpClient.newHttpClient();
        this.gson = new Gson();
    }

    public void startAnalysis(IProject project, String apiKey) {
        Job job = new Job("Code Historian Analysis") {
            @Override
            protected IStatus run(IProgressMonitor monitor) {
                try {
                    String projectPath = project.getLocation().toOSString();
                    String requestBody = gson.toJson(new AnalysisRequest(projectPath));

                    HttpRequest request = HttpRequest.newBuilder()
                        .uri(URI.create(API_URL + "/analysis/start"))
                        .header("Content-Type", "application/json")
                        .header("X-API-Key", apiKey)
                        .POST(HttpRequest.BodyPublishers.ofString(requestBody))
                        .build();

                    CompletableFuture<HttpResponse<String>> response = client.sendAsync(
                        request,
                        HttpResponse.BodyHandlers.ofString()
                    );

                    response.thenAccept(res -> {
                        if (res.statusCode() == 200) {
                            // Analysis started successfully
                            connectWebSocket(gson.fromJson(res.body(), AnalysisResponse.class).id);
                        }
                    });

                    return Status.OK_STATUS;
                } catch (Exception e) {
                    return new Status(IStatus.ERROR, "io.codehistorian.eclipse", "Analysis failed", e);
                }
            }
        };
        job.schedule();
    }

    private void connectWebSocket(String analysisId) {
        // WebSocket connection implementation
    }

    private static class AnalysisRequest {
        private final String projectPath;

        public AnalysisRequest(String projectPath) {
            this.projectPath = projectPath;
        }
    }

    private static class AnalysisResponse {
        private String id;
    }
} 