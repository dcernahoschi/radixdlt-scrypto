use sbor::{Decode, Encode};

use crate::buffer::*;
use crate::kernel::*;
use crate::types::rust::string::String;
use crate::types::rust::string::ToString;
use crate::types::rust::vec::Vec;
use crate::types::*;

/// A self-executing program that holds resources and exposed actions to other entities.
#[derive(Debug)]
pub struct Component {
    address: Address,
}

impl From<Address> for Component {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl Component {
    pub fn new<T: Encode>(name: &str, state: T) -> Address {
        let input = CreateComponentInput {
            name: name.to_string(),
            state: scrypto_encode(&state),
        };
        let output: CreateComponentOutput = call_kernel(CREATE_COMPONENT, input);

        output.component
    }

    pub fn call(&self, method: &str, args: Vec<Vec<u8>>) -> Vec<u8> {
        let data = self.get_info();

        let mut args_buf = Vec::new();
        args_buf.push(scrypto_encode(&self.address));
        args_buf.extend(args);

        let input = CallBlueprintInput {
            blueprint: data.blueprint,
            component: data.name,
            method: method.to_string(),
            args: args_buf,
        };
        let output: CallBlueprintOutput = call_kernel(CALL_BLUEPRINT, input);

        output.rtn
    }

    pub fn get_info(&self) -> ComponentInfo {
        let input = GetComponentInfoInput {
            component: self.address,
        };
        let output: GetComponentInfoOutput = call_kernel(GET_COMPONENT_INFO, input);

        output.result.unwrap()
    }

    pub fn get_blueprint(&self) -> Address {
        self.get_info().blueprint
    }

    pub fn get_name(&self) -> String {
        self.get_info().name
    }

    pub fn get_state<T: Decode>(&self) -> T {
        let input = GetComponentStateInput {
            component: self.address,
        };
        let output: GetComponentStateOutput = call_kernel(GET_COMPONENT_STATE, input);

        scrypto_decode(&output.state).unwrap()
    }

    pub fn put_state<T: Encode>(&self, state: T) {
        let input = PutComponentStateInput {
            component: self.address,
            state: scrypto_encode(&state),
        };
        let _: PutComponentStateOutput = call_kernel(PUT_COMPONENT_STATE, input);
    }

    pub fn address(&self) -> Address {
        self.address
    }
}
