use std::future::Future;

use neural0_assembler::SimpleAssembler;
use neural0_kernel::module::extract_code;
use neural0_kernel::VM;
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
    ServerHandler, ServiceExt,
};
use serde::Deserialize;

#[derive(Debug, Clone, Default)]
pub struct N0McpServer {
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AssembleRequest {
    #[schemars(description = "NEURAL-0 assembly source code as a string")]
    pub source: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunRequest {
    #[schemars(description = "Hex-encoded .n0b module (with or without 0x prefix)")]
    pub binary: String,
}

#[tool_router]
impl N0McpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "assemble",
        description = "Assemble NEURAL-0 source (.n0asm) into a hex-encoded .n0b module. \
                      The full assembler supports only a small subset of mnemonics in v1 \
                      (PUSH_I64, ADD, HALT); richer mnemonics will be added as Phase 1 progresses."
    )]
    async fn assemble(
        &self,
        Parameters(AssembleRequest { source }): Parameters<AssembleRequest>,
    ) -> Result<String, String> {
        let bytes =
            SimpleAssembler::assemble(&source).map_err(|e| format!("assembly error: {}", e))?;
        Ok(hex_encode(&bytes))
    }

    #[tool(
        name = "run",
        description = "Run a hex-encoded NEURAL-0 module on a fresh VM and return the final stack. \
                      Output is a string like 'halted=true\\nstack=[I64(5)]' on success, \
                      or a trap message on failure."
    )]
    async fn run(
        &self,
        Parameters(RunRequest { binary }): Parameters<RunRequest>,
    ) -> Result<String, String> {
        let bytes = hex_decode(&binary)?;
        let code = extract_code(&bytes).map_err(|e| format!("invalid module: {}", e))?;
        let mut vm = VM::new(1024, 65536);
        match vm.load_and_run(code) {
            Ok(()) => Ok(format!(
                "halted={}\nyielded={}\nstack={:?}",
                vm.is_halted(),
                vm.has_yielded(),
                vm.stack()
            )),
            Err(e) => Err(format!("VM trapped: {:?}", e)),
        }
    }
}

#[tool_handler]
impl ServerHandler for N0McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "NEURAL-0 MCP server. Use the `assemble` tool to compile .n0asm to .n0b, \
                 then `run` to execute on a fresh VM and read the final stack. \
                 See AGENTS.md and CONTRIBUTING.md in the project root for the full agent contract."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = N0McpServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| eprintln!("Error starting n0-mcp server: {}", e))?;
    service.waiting().await?;
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim().trim_start_matches("0x");
    if !s.len().is_multiple_of(2) {
        return Err("hex string must have an even number of characters".to_string());
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let mut i = 0;
    while i < s.len() {
        let pair = &s[i..i + 2];
        let byte = u8::from_str_radix(pair, 16)
            .map_err(|e| format!("invalid hex at offset {}: {}", i, e))?;
        out.push(byte);
        i += 2;
    }
    Ok(out)
}
