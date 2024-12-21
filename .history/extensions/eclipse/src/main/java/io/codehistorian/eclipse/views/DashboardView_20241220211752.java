package io.codehistorian.eclipse.views;

import org.eclipse.swt.SWT;
import org.eclipse.swt.browser.Browser;
import org.eclipse.swt.layout.GridData;
import org.eclipse.swt.layout.GridLayout;
import org.eclipse.swt.widgets.Composite;
import org.eclipse.ui.part.ViewPart;

public class DashboardView extends ViewPart {
    public static final String ID = "io.codehistorian.eclipse.views.DashboardView";
    private Browser browser;

    @Override
    public void createPartControl(Composite parent) {
        Composite container = new Composite(parent, SWT.NONE);
        container.setLayout(new GridLayout(1, false));

        browser = new Browser(container, SWT.NONE);
        browser.setLayoutData(new GridData(GridData.FILL_BOTH));

        // Load dashboard HTML
        String dashboardHtml = generateDashboardHtml();
        browser.setText(dashboardHtml);
    }

    @Override
    public void setFocus() {
        browser.setFocus();
    }

    private String generateDashboardHtml() {
        return """
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <title>Code Historian Dashboard</title>
                <style>
                    body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }
                    .dashboard-grid {
                        display: grid;
                        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                        gap: 20px;
                    }
                    .card {
                        background: #fff;
                        border: 1px solid #ddd;
                        border-radius: 4px;
                        padding: 15px;
                    }
                    .metric {
                        font-size: 24px;
                        font-weight: bold;
                        color: #2196F3;
                    }
                </style>
                <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
            </head>
            <body>
                <div class="dashboard-grid">
                    <div class="card">
                        <h2>Analysis Progress</h2>
                        <div class="progress-bar">
                            <div class="progress-fill" style="width: 0%"></div>
                        </div>
                    </div>
                    <div class="card">
                        <h2>Repository Overview</h2>
                        <div id="repoMetrics"></div>
                    </div>
                    <div class="card">
                        <h2>Code Churn</h2>
                        <canvas id="churnChart"></canvas>
                    </div>
                    <div class="card">
                        <h2>Impact Analysis</h2>
                        <canvas id="impactChart"></canvas>
                    </div>
                </div>
                <script>
                    // Dashboard JavaScript implementation
                </script>
            </body>
            </html>
        """;
    }

    public void updateProgress(double progress) {
        if (!browser.isDisposed()) {
            browser.execute(String.format("updateProgress(%f)", progress));
        }
    }

    public void updateMetrics(String metricsJson) {
        if (!browser.isDisposed()) {
            browser.execute(String.format("updateMetrics(%s)", metricsJson));
        }
    }
} 