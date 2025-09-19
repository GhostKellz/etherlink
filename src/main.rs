use etherlink::{EtherlinkClient, EtherlinkClientBuilder, CNSClient, GhostPlaneClient};
use tracing::{info, error};

#[tokio::main]
async fn main() -> etherlink::Result<()> {
    // Initialize tracing
    etherlink::init_with_tracing("etherlink=debug")?;

    info!("Starting Etherlink client demo");

    // Create Etherlink client
    let mut client = EtherlinkClientBuilder::new()
        .ghostd_endpoint("http://localhost:8545")
        .enable_tls(false)
        .timeout_ms(30000)
        .build();

    // Connect to GhostChain
    match client.connect().await {
        Ok(_) => info!("Successfully connected to GhostChain"),
        Err(e) => {
            error!("Failed to connect to GhostChain: {}", e);
            info!("This is expected if ghostd is not running");
        }
    }

    // Create CNS client
    let cns_client = CNSClient::with_defaults();
    match cns_client.connect().await {
        Ok(_) => info!("Successfully connected to CNS"),
        Err(e) => {
            error!("Failed to connect to CNS: {}", e);
            info!("This is expected if CNS service is not running");
        }
    }

    // Create GhostPlane client
    let mut ghostplane_client = GhostPlaneClient::with_defaults();
    match ghostplane_client.initialize().await {
        Ok(_) => info!("Successfully initialized GhostPlane client"),
        Err(e) => {
            error!("Failed to initialize GhostPlane: {}", e);
            info!("This is expected if GhostPlane is not available");
        }
    }

    // Demonstrate basic functionality
    demo_basic_functionality().await?;

    info!("Etherlink client demo completed");
    Ok(())
}

async fn demo_basic_functionality() -> etherlink::Result<()> {
    info!("Running basic functionality demo");

    // Test CNS domain resolution
    let cns = CNSClient::with_defaults();
    match cns.resolve_domain("example.ghost").await {
        Ok(resolution) => info!("Resolved domain: {} -> {}", resolution.domain, resolution.owner),
        Err(e) => info!("Domain resolution failed (expected): {}", e),
    }

    // Test domain availability check
    match cns.is_domain_available("test.ghost").await {
        Ok(available) => info!("Domain test.ghost available: {}", available),
        Err(e) => info!("Availability check failed: {}", e),
    }

    // Test GhostPlane state query
    let ghostplane = GhostPlaneClient::with_defaults();
    match ghostplane.query_state("block_height").await {
        Ok(state) => info!("GhostPlane state: {}", state),
        Err(e) => info!("State query failed (expected): {}", e),
    }

    info!("Basic functionality demo completed");
    Ok(())
}
