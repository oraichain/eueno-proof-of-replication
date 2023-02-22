#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{SETUP_PARAMS, OWNER};
use contract_auxiliaries::deserializer::{deserialize_verifying_key, deserialize_proof};
use contract_auxiliaries::drg::stacked::VerifierStackedDrg;
use contract_auxiliaries::domain::{poseidon::PoseidonDomain, sha256::Sha256Domain};

// version info for migration info
const CONTRACT_NAME: &str = "stacked-drg";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION"); 

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    OWNER.save(deps.storage, &info.sender)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetSetupParams {setup_params_json} => execute::set_setup_params(deps, info, setup_params_json)
    }
}

pub mod execute {

    use super::*;
    pub fn set_setup_params(deps: DepsMut, info: MessageInfo, setup_params_json: String) -> Result<Response, ContractError>{
        if info.sender == OWNER.load(deps.storage).unwrap() {
            SETUP_PARAMS.save(deps.storage, &setup_params_json)?;
            Ok(Response::default())
        }
        else {
            Err(ContractError::Unauthorized{})
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VerifyProofJson {vk_raw, proof_raw, public_inputs_json} => to_binary(&query::verify_proof_json(deps, vk_raw, proof_raw, public_inputs_json)?)
    }
}

pub mod query { 

    use super::*;
    pub fn verify_proof_json(
        deps: Deps, 
        vk_raw: Vec<u8>,
        proof_raw: Vec<u8>,
        public_inputs_json: String,
    ) -> StdResult<bool>
    {
        let vk = deserialize_verifying_key(&vk_raw).unwrap();
        let proof = deserialize_proof(&proof_raw).unwrap();
        let public_inputs = serde_json::from_str(&public_inputs_json).unwrap();
        let setup_params_json = SETUP_PARAMS.load(deps.storage).unwrap();
        let setup_params = serde_json::from_str(&setup_params_json).unwrap();
        let public_params = VerifierStackedDrg::<PoseidonDomain, Sha256Domain>::setup(&setup_params).unwrap();
        let verifier = VerifierStackedDrg::<PoseidonDomain, Sha256Domain>::new(&vk);
        let verified = verifier.verify(
            &public_params,
            &public_inputs,
            &proof,
            &contract_auxiliaries::drg::stacked::challenges::ChallengeRequirements {
                minimum_challenges: 1,
            },
        );
        Ok(true)
    }
}
