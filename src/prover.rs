use bitcoincore_rpc::bitcoin;
use rand::rngs::OsRng;
use rand::RngCore;
use secp256k1::{Secp256k1, SecretKey, PublicKey, XOnlyPublicKey, schnorr::Signature, Message, Keypair};
use crate::challenge::BulletproofChallenge;
use crate::confidential_tx::{create_confidential_tx, ConfidentialTransaction};
use crate::inner_product::compute_inner_product_commitment;
use crate::network::send_proof;
use serde_json;
use std::error::Error;

pub struct BulletproofResponse {
    pub response_commitment: PublicKey,
    pub response_challenge: SecretKey,
}

// Responds to the challenge during proof generation
pub fn respond_to_challenge(
    bit_commitments: &[PublicKey],
    challenge: &BulletproofChallenge,
) -> BulletproofResponse {
    let secp = Secp256k1::new();

    let response_commitment = compute_inner_product_commitment(bit_commitments);

    let mut rng = OsRng;
    let mut response_bytes = [0u8; 32];
    rng.fill_bytes(&mut response_bytes);
    let response_challenge = SecretKey::from_slice(&response_bytes).expect("Response failed");

    BulletproofResponse {
        response_commitment,
        response_challenge,
    }
}

// Generates a new key pair (Secret & Public Key)
fn generate_keypair(secp: &Secp256k1<secp256k1::All>, rng: &mut OsRng) -> (SecretKey, PublicKey) {
    let sk = SecretKey::new(rng);
    let pk = PublicKey::from_secret_key(secp, &sk);
    (sk, pk)
}

// Main Prover Function
pub async fn prover_main() -> Result<(), Box<dyn Error>> {
    let secp = Secp256k1::new();
    let mut rng = OsRng;

    // Generate sender & receiver keys
    let (sender_sk, sender_pk) = generate_keypair(&secp, &mut rng);
    let sender_keypair = Keypair::from_secret_key(&secp, &sender_sk);
    let sender_xonly_pk = sender_keypair.x_only_public_key().0;
    
    let (receiver_sk, receiver_pk) = generate_keypair(&secp, &mut rng);

    println!("Prover: Creating Confidential Transaction...");
    let mut tx = create_confidential_tx(&sender_sk, &receiver_pk, 42);

    // Store the XOnlyPublicKey as bytes in transaction
    tx.sender = sender_xonly_pk.serialize().to_vec();

    // Create Schnorr Signature
    let tx_bytes = serde_json::to_vec(&tx)?;
    let tx_hash = Message::from_hashed_data::<secp256k1::hashes::sha256::Hash>(&tx_bytes);
    let schnorr_sig = secp.sign_schnorr_with_rng(&tx_hash, &sender_keypair, &mut rng);

    println!("Prover: Signing transaction with Schnorr Signature...");
    
    let signed_tx = (tx, schnorr_sig.serialize().to_vec());

    // Convert to JSON and send to verifier
    let tx_json = serde_json::to_string(&signed_tx)?;
    send_proof(tx_json, "127.0.0.1:8080").await;

    Ok(())
}
