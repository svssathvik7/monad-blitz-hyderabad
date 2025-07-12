use std::{collections::HashMap, sync::Arc, time::Duration};

use eyre::Result;
use tokio::runtime::Handle;
use tracing::{
    field::{Field, Visit},
    Event, Subscriber,
};
use tracing_subscriber::{layer::Context, Layer};

#[derive(Clone)]
pub struct WebhookLayer {
    webhook_url: String,
    http_client: reqwest::Client,
    name: String,
    level: tracing::Level,
    formatter: Arc<dyn Fn(&Event, &FieldVisitor, &str) -> serde_json::Value + Send + Sync>,
}

/// Field visitor that collects field values from tracing events.
///
/// This visitor implements the tracing `Visit` trait to collect field values
/// from log events into a hashmap for later formatting and display.
pub struct FieldVisitor {
    fields: HashMap<String, String>,
}

impl FieldVisitor {
    /// Creates a new empty field visitor.
    fn new() -> Self {
        Self {
            fields: HashMap::with_capacity(10), // Pre-allocate with reasonable capacity
        }
    }
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields
            .insert(field.name().to_string(), format!("{:?}", value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    // Additional implementations for other field types to improve data capture quality
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.fields
            .insert(field.name().to_string(), format!("Error: {}", value));
    }
}

impl<S: Subscriber> Layer<S> for WebhookLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Early return if the event level is above the configured level
        if *event.metadata().level() > self.level {
            return;
        }

        // Collect all fields from the event
        let mut visitor = FieldVisitor::new();
        event.record(&mut visitor);

        let message = (self.formatter)(event, &visitor, &self.name);

        let webhook_url = self.webhook_url.clone();
        let client = self.http_client.clone();

        // Spawning the webhook sending as a non-blocking task
        let rt = Handle::current();
        rt.spawn(async move {
            // Use a timeout to prevent hanging if the webhook is slow to respond
            let result = tokio::time::timeout(
                Duration::from_secs(5),
                client.post(webhook_url.as_str()).json(&message).send(),
            )
            .await;

            match result {
                Ok(Ok(response)) => {
                    if !response.status().is_success() {
                        eprintln!("Webhook error: HTTP {}", response.status());
                    }
                }
                Ok(Err(e)) => eprintln!("Failed to send to webhook: {}", e),
                Err(_) => eprintln!("Webhook request timed out"),
            }
        });
    }
}

impl WebhookLayer {
    /// Creates a new webhook layer with optional rate limiting.
    ///
    /// # Arguments
    ///
    /// * `webhook_url` - The webhook URL to send events to
    /// * `name` - Name to display for the webhook message
    /// * `level` - The tracing level threshold for sending events
    /// * `formatter` - Custom formatter function for webhook messages
    ///
    /// # Returns
    ///
    /// A new `WebhookLayer` instance or an error if initialization fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tracing::Level;
    /// use utils::WebhookLayer;
    /// use utils::default_message_formatter;
    ///
    /// // Without rate limiting
    /// let discord_layer = WebhookLayer::new(
    ///     "https://discord.com/api/webhooks/...",
    ///     "MyApp",
    ///     Level::ERROR,
    ///     |event, visitor, name| {
    ///         // Custom formatting logic
    ///         serde_json::json!({
    ///             "content": format!("Error in {}", event.metadata().target())
    ///         })
    ///     }
    /// ).unwrap();
    ///
    /// let discord_layer = WebhookLayer::new(
    ///     "https://discord.com/api/webhooks/...",
    ///     "MyApp",
    ///     Level::ERROR,
    ///     default_message_formatter
    /// ).unwrap();
    /// ```
    pub fn new<F>(
        webhook_url: &str,
        name: &str,
        level: tracing::Level,
        formatter: F,
    ) -> Result<Self>
    where
        F: Fn(&Event, &FieldVisitor, &str) -> serde_json::Value + Send + Sync + 'static,
    {
        // Validate the webhook URL format
        if !webhook_url.starts_with("https://") {
            return Err(eyre::eyre!("Invalid webhook URL format"));
        }

        // HTTP client with timeout and user agent
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Rust-WebhookLayer/1.0")
            .build()
            .map_err(|e| eyre::eyre!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            webhook_url: webhook_url.to_string(),
            http_client: client,
            name: name.to_string(),
            level,
            formatter: Arc::new(formatter),
        })
    }
}

/// Default message formatter
///
/// Creates a standardized Discord-friendly error message
pub fn default_message_formatter(
    event: &Event,
    visitor: &FieldVisitor,
    name: &str,
) -> serde_json::Value {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let target = event.metadata().target();
    let target_parts: Vec<&str> = target.split(':').collect();
    let short_target = target_parts.last().unwrap_or(&target);

    let mut content = String::new();
    content.push_str(&format!("Target: {}\n", short_target));

    // Add location info if available
    if let Some(file) = event.metadata().file() {
        if let Some(line) = event.metadata().line() {
            content.push_str(&format!("Location: {}:{}\n", file, line));
        }
    }

    for (key, value) in &visitor.fields {
        content.push_str(&format!("{}: {}\n", key, value));
    }

    let color = match *event.metadata().level() {
        tracing::Level::ERROR => 0xFF0000, // Red
        tracing::Level::WARN => 0xFFA500,  // Orange
        tracing::Level::INFO => 0x00FF00,  // Green
        tracing::Level::DEBUG => 0x808080, // Gray
        tracing::Level::TRACE => 0x0000FF, // Blue
    };

    serde_json::json!({
        "username": name,
        "embeds": [{
            "title": format!("{} event from {}", event.metadata().level().to_string(), name),
            "description": content,
            "color": color,
            "timestamp": timestamp
        }]
    })
}

/// Sets up tracing with a webhook for notifications.
///
/// This function configures a tracing subscriber with:
/// 1. A standard formatting layer for console output (INFO level and above)
/// 2. A webhook layer for sending events at the specified level
///
/// # Arguments
///
/// * `webhook_url` - The webhook URL to send events to
/// * `app_name` - Name to display for the webhook messages
/// * `level` - The level of events to send to the webhook (e.g., ERROR)
/// * `rate_limit_per_minute` - Optional rate limit for webhook messages
/// * `formatter` - Optional custom formatter function for webhook messages
///
/// # Returns
///
/// `Result<()>` - Ok if initialization succeeded, or an error
///
/// # Examples
///
/// ```no_run
/// use tracing::Level;
/// let webhook_url = std::env::var("WEBHOOK_URL")
///     .unwrap_or_else(|_| "https://discord.com/api/webhooks/your/url".to_string());
///
/// utils::setup_tracing_with_webhook(
///     &webhook_url,
///     "MyApp",
///     Level::ERROR,
///     None
/// ).unwrap();
///
/// // Now events at ERROR level will be sent to the webhook
/// tracing::error!("This will appear in the webhook");
/// ```
pub fn setup_tracing_with_webhook(
    webhook_url: &str,
    app_name: &str,
    level: tracing::Level,
    formatter: Option<Box<dyn Fn(&Event, &FieldVisitor, &str) -> serde_json::Value + Send + Sync>>,
) -> Result<()> {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(tracing_subscriber::filter::LevelFilter::INFO);

    use tracing_subscriber::prelude::*;

    let webhook_layer = WebhookLayer::new(
        webhook_url,
        app_name,
        level,
        match formatter {
            Some(f) => f,
            None => Box::new(default_message_formatter),
        },
    )?;

    let subscriber = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(webhook_layer);

    subscriber
        .try_init()
        .map_err(|e| eyre::eyre!("Failed to initialize tracing: {}", e))
}
