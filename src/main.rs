use structopt::StructOpt;
use hyper_tls::HttpsConnector;
use hyper::{Body, Method, Client, Request};
use serde::{Deserialize, Serialize};
use dotenv;

/// Post a CASD request to a Teams channel.
#[derive(StructOpt, Debug)]
#[structopt(name = "casdtt", author)]
struct Opt {
    // A flag, true if used in the command line.
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,
    /// Service Desk request number
    #[structopt(short, long)]
    id: u32,
    /// Title to display on Teams's message
    #[structopt(short, long)]
    title: String,
    /// Summary of request
    #[structopt(short, long)]
    summary: Option<String>,
}

fn default_type() -> String {
    "AdaptiveCard".to_string()
}

#[derive(Serialize, Deserialize)]
struct ServiceRequest {
    #[serde(default="default_type")]
    r#type: String,
    title: String,
    text: String,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    // read "CASD_URL" environment variable
    dotenv::dotenv().ok();
    
    let casd_uri = std::env::var("CASD_WEBHOOK_URL")
            .expect("CASD_WEBHOOK_URL must be set.");

    // get command line options
    let opt = Opt::from_args();

    if opt.debug {
        println!("CASD_WEBHOOK_URL: {}\n", casd_uri);
        println!("DEBUG (Opt):\n{:#?}\n", opt);
    }

    let service_request = ServiceRequest {
        r#type: "AdaptiveCard".to_string(),
        title: format!("SR {} - {}", opt.id, opt.title),
        text: match opt.summary {
            Some(v) => v,
            None => format!("{}", "No summary provided."),
        },
    };

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);


    // hard-coded example of adaptive card
    let adaptive_card = format!(r#"{{
    "type":"message",
    "attachments": [
        {{
            "contentType":"application/vnd.microsoft.card.adaptive",
            "contentUrl":null,
            "content": {{
                "$schema":"http://adaptivecards.io/schemas/adaptive-card.json",
                "type":"AdaptiveCard",
                "version":"1.2",
                "body": [
                    {{
                        "type": "TextBlock",
                        "text": "{body_title}",
                        "size": "large",
                        "weight": "bolder",
                        "color": "accent"
                    }},
                    {{
                        "type": "TextBlock",
                        "text": "{body_summary}",
                        "wrap": "true",
                    }}
                ],
                "msteams": {{
                    "width": "Full"
                }},
            }}
        }}
    ]
}}"#, body_title=service_request.title, body_summary=service_request.text);
    
    if opt.debug {
        println!("DEBUG (adaptive_card):\n\n{}", adaptive_card);
    }
    
 
    let req = Request::builder()
        .method(Method::POST)
        .uri(casd_uri)
        .header("Content-Type", "application/json")
        .body(Body::from(adaptive_card))?;

    // POST it...
    let resp = client.request(req).await?;

    println!("Response: {}", resp.status());

    Ok(())
}
