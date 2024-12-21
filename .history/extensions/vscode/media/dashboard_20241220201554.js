// Initialize charts
let progressChart, activityChart, comparisonChart;

document.addEventListener('DOMContentLoaded', () => {
    initializeCharts();
    setupMessageHandler();
});

function initializeCharts() {
    // Progress Chart
    const progressCtx = document.getElementById('progressChart').getContext('2d');
    progressChart = new Chart(progressCtx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [{
                label: 'Analysis Progress',
                data: [],
                borderColor: getComputedStyle(document.documentElement)
                    .getPropertyValue('--vscode-charts-blue'),
                tension: 0.1
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                y: {
                    beginAtZero: true,
                    max: 100,
                    grid: {
                        color: getComputedStyle(document.documentElement)
                            .getPropertyValue('--vscode-charts-lines')
                    }
                },
                x: {
                    grid: {
                        color: getComputedStyle(document.documentElement)
                            .getPropertyValue('--vscode-charts-lines')
                    }
                }
            },
            plugins: {
                legend: {
                    labels: {
                        color: getComputedStyle(document.documentElement)
                            .getPropertyValue('--vscode-foreground')
                    }
                }
            }
        }
    });

    // Activity Chart
    const activityCtx = document.getElementById('activityChart').getContext('2d');
    activityChart = new Chart(activityCtx, {
        type: 'bar',
        data: {
            labels: [],
            datasets: [{
                label: 'Team Activity',
                data: [],
                backgroundColor: getComputedStyle(document.documentElement)
                    .getPropertyValue('--vscode-charts-purple')
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                y: {
                    beginAtZero: true,
                    grid: {
                        color: getComputedStyle(document.documentElement)
                            .getPropertyValue('--vscode-charts-lines')
                    }
                },
                x: {
                    grid: {
                        color: getComputedStyle(document.documentElement)
                            .getPropertyValue('--vscode-charts-lines')
                    }
                }
            },
            plugins: {
                legend: {
                    labels: {
                        color: getComputedStyle(document.documentElement)
                            .getPropertyValue('--vscode-foreground')
                    }
                }
            }
        }
    });

    // Comparison Chart
    const comparisonCtx = document.getElementById('comparisonChart').getContext('2d');
    comparisonChart = new Chart(comparisonCtx, {
        type: 'radar',
        data: {
            labels: ['Code Changes', 'Impact', 'Complexity', 'Team Size', 'Activity'],
            datasets: []
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                r: {
                    beginAtZero: true,
                    grid: {
                        color: getComputedStyle(document.documentElement)
                            .getPropertyValue('--vscode-charts-lines')
                    }
                }
            },
            plugins: {
                legend: {
                    labels: {
                        color: getComputedStyle(document.documentElement)
                            .getPropertyValue('--vscode-foreground')
                    }
                }
            }
        }
    });
}

function setupMessageHandler() {
    window.addEventListener('message', event => {
        const message = event.data;
        switch (message.type) {
            case 'update':
                updateDashboard(message.data);
                break;
            case 'error':
                showError(message.data);
                break;
        }
    });
}

function updateDashboard(data) {
    // Update metrics
    if (data.metrics) {
        document.getElementById('activeAnalyses').textContent = data.metrics.active_analyses;
        document.getElementById('teamMembers').textContent = data.metrics.total_team_members;
        document.getElementById('totalProjects').textContent = data.metrics.total_projects;
        document.getElementById('avgImpactScore').textContent = 
            data.metrics.avg_impact_score.toFixed(2);
    }

    // Update progress chart
    if (data.progress) {
        updateProgressChart(data.progress);
    }

    // Update activity chart
    if (data.activity) {
        updateActivityChart(data.activity);
    }

    // Update comparison chart
    if (data.comparison) {
        updateComparisonChart(data.comparison);
    }

    // Update activity feed
    if (data.recent_activity) {
        updateActivityFeed(data.recent_activity);
    }
}

function updateProgressChart(progress) {
    const chart = progressChart;
    chart.data.labels.push(moment().format('HH:mm:ss'));
    chart.data.datasets[0].data.push(progress);

    // Keep last 20 data points
    if (chart.data.labels.length > 20) {
        chart.data.labels.shift();
        chart.data.datasets[0].data.shift();
    }

    chart.update();
}

function updateActivityChart(activity) {
    const chart = activityChart;
    chart.data.labels = activity.map(a => a.label);
    chart.data.datasets[0].data = activity.map(a => a.value);
    chart.update();
}

function updateComparisonChart(comparison) {
    const chart = comparisonChart;
    chart.data.datasets = comparison.projects.map((project, index) => ({
        label: project.name,
        data: project.metrics,
        fill: true,
        backgroundColor: getComputedStyle(document.documentElement)
            .getPropertyValue(`--vscode-charts-${index % 5}`),
        borderColor: getComputedStyle(document.documentElement)
            .getPropertyValue(`--vscode-charts-${index % 5}`)
    }));
    chart.update();
}

function updateActivityFeed(activities) {
    const feed = document.getElementById('activityList');
    activities.forEach(activity => {
        const item = document.createElement('div');
        item.className = 'activity-item';
        item.innerHTML = `
            <div class="activity-content">
                <div class="activity-title">${activity.action}</div>
                <div class="activity-details">${activity.details}</div>
            </div>
            <div class="activity-time">${moment(activity.timestamp).fromNow()}</div>
        `;
        feed.insertBefore(item, feed.firstChild);
    });

    // Keep last 10 activities
    while (feed.children.length > 10) {
        feed.removeChild(feed.lastChild);
    }
}

function showError(message) {
    // Add error notification to the dashboard
    const container = document.querySelector('.container');
    const error = document.createElement('div');
    error.className = 'error-message';
    error.textContent = message;
    container.insertBefore(error, container.firstChild);

    // Remove after 5 seconds
    setTimeout(() => {
        error.remove();
    }, 5000);
}

// Theme handling
function updateTheme() {
    const isDark = document.body.classList.contains('vscode-dark');
    Chart.defaults.color = getComputedStyle(document.documentElement)
        .getPropertyValue('--vscode-foreground');
    Chart.defaults.borderColor = getComputedStyle(document.documentElement)
        .getPropertyValue('--vscode-charts-lines');

    [progressChart, activityChart, comparisonChart].forEach(chart => {
        if (chart) {
            chart.update();
        }
    });
}

// Listen for theme changes
const observer = new MutationObserver(mutations => {
    mutations.forEach(mutation => {
        if (mutation.attributeName === 'class') {
            updateTheme();
        }
    });
});

observer.observe(document.body, {
    attributes: true
});

// Initial theme setup
updateTheme(); 