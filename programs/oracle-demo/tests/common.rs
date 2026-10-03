use std::{env, error::Error, path::Path, rc::Rc};

use anchor_client::{Client, Cluster};
use anchor_lang::InstructionData;
use base64::{engine::general_purpose::STANDARD, Engine};
use oracle_demo::SCALE;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::{read_keypair_file, Keypair};
use solana_pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;
use solana_signer::Signer;
use solana_transaction::Transaction;

pub type TestResult<T> = Result<T, Box<dyn Error>>;

pub struct TestContext {
    anchor_client: Client<Rc<Keypair>>,
    client: RpcClient,
    payer: Rc<Keypair>,
    program_id: Pubkey,
}

impl TestContext {
    pub fn new() -> TestResult<Self> {
        dotenvy::dotenv().ok();
        let rpc_url = required("SOLANA_RPC_URL")?;
        let keypair_path = required("KEYPAIR_PATH")?;
        let program_keypair_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/deploy/oracle_demo-keypair.json");
        let program_id = read_keypair_file(program_keypair_path)?.pubkey();
        let payer = Rc::new(read_keypair_file(Path::new(&keypair_path))?);
        let anchor_client = Client::new(
            Cluster::Custom(rpc_url.clone(), rpc_url.clone()),
            Rc::clone(&payer),
        );

        Ok(Self {
            anchor_client,
            client: RpcClient::new(rpc_url),
            payer,
            program_id,
        })
    }

    pub fn payer(&self) -> Pubkey {
        self.payer.pubkey()
    }

    pub fn invoke<T: InstructionData>(
        &self,
        data: T,
        accounts: Vec<AccountMeta>,
    ) -> TestResult<u128> {
        self.invoke_with_prefix(Vec::new(), data, accounts)
    }

    pub fn invoke_with_prefix<T: InstructionData>(
        &self,
        mut prefix: Vec<Instruction>,
        data: T,
        accounts: Vec<AccountMeta>,
    ) -> TestResult<u128> {
        let program = self.anchor_client.program(self.program_id)?;
        let instruction = program
            .request()
            .accounts(accounts)
            .args(data)
            .instructions()
            .pop()
            .ok_or("Anchor Client did not build an instruction")?;
        prefix.push(instruction);
        let transaction = Transaction::new_signed_with_payer(
            &prefix,
            Some(&self.payer()),
            &[self.payer.as_ref()],
            self.client.get_latest_blockhash()?,
        );
        let simulation = self.client.simulate_transaction(&transaction)?;
        if let Some(error) = simulation.value.err {
            return Err(format!(
                "transaction simulation failed: {error:?}\nlogs: {:?}",
                simulation.value.logs
            )
            .into());
        }
        let return_data = simulation
            .value
            .return_data
            .ok_or("oracle-demo did not set return data")?;
        if return_data.program_id != self.program_id.to_string() {
            return Err(format!(
                "oracle-demo return data came from {}, expected {}",
                return_data.program_id, self.program_id
            )
            .into());
        }
        let bytes = STANDARD.decode(return_data.data.0)?;
        let price_bytes: [u8; 16] = bytes
            .as_slice()
            .try_into()
            .map_err(|_| "oracle-demo returned a price with an unexpected length")?;
        let price = u128::from_le_bytes(price_bytes);
        if price == 0 {
            return Err("oracle-demo returned a zero price".into());
        }
        self.client.send_and_confirm_transaction(&transaction)?;
        Ok(price)
    }
}

pub fn required(name: &str) -> TestResult<String> {
    env::var(name).map_err(|_| format!("{name} must be set for this devnet test").into())
}

pub fn feed_id_hex(value: &str) -> TestResult<[u8; 32]> {
    let value = value.strip_prefix("0x").unwrap_or(value);
    let bytes = hex::decode(value)?;
    Ok(bytes
        .as_slice()
        .try_into()
        .map_err(|_| "feed ID must contain exactly 32 bytes of hexadecimal data")?)
}

pub fn format_ui_price(price: u128) -> String {
    let scale = u128::from(SCALE);
    let whole = price / scale;
    let fraction = price % scale;
    if fraction == 0 {
        return whole.to_string();
    }

    let fraction = format!("{fraction:012}");
    format!("{whole}.{}", fraction.trim_end_matches('0'))
}
