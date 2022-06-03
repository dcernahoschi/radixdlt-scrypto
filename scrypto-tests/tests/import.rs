#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::borrow::ToOwned;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::component::*;
use scrypto::{blueprint, import};

// base directory: `scrypto-derive`
import! {
r#"
{
    "package_address": "056967d3d49213394892980af59be76e9b3e7cc4cb78237460d0c7",
    "blueprint_name": "Simple",
    "functions": [
        {
            "name": "stateless_func",
            "inputs": [],
            "output": {
                "type": "U32"
            }
        },
        {
            "name": "test_custom_types",
            "inputs": [
                {
                    "type": "Custom",
                    "type_id": 161,
                    "generics": []
                },
                {
                    "type": "Custom",
                    "type_id": 128,
                    "generics": []
                },
                {
                    "type": "Custom",
                    "type_id": 129,
                    "generics": []
                },
                {
                    "type": "Custom",
                    "type_id": 130,
                    "generics": [
                        {
                            "type": "String"
                        },
                        {
                            "type": "String"
                        }
                    ]
                },
                {
                    "type": "Custom",
                    "type_id": 177,
                    "generics": []
                },
                {
                    "type": "Custom",
                    "type_id": 178,
                    "generics": []
                },
                {
                    "type": "Custom",
                    "type_id": 179,
                    "generics": []
                },
                {
                    "type": "Custom",
                    "type_id": 182,
                    "generics": []
                }
            ],
            "output": {
                "type": "Custom",
                "type_id": 177,
                "generics": []
            }
        }
    ],
    "methods": [
        {
            "name": "calculate_volume",
            "mutability": "Immutable",
            "inputs": [
                {
                    "type": "Struct",
                    "name": "Floor",
                    "fields": {
                        "type": "Named",
                        "named": [
                            [
                                "x",
                                {
                                    "type": "U32"
                                }
                            ],
                            [
                                "y",
                                {
                                    "type": "U32"
                                }
                            ]
                        ]
                    }
                },
                {
                    "type": "Tuple",
                    "elements": [
                        {
                            "type": "U8"
                        },
                        {
                            "type": "U16"
                        }
                    ]
                },
                {
                    "type": "Vec",
                    "element": {
                        "type": "String"
                    }
                },
                {
                    "type": "U32"
                },
                {
                    "type": "Enum",
                    "name": "Hello",
                    "variants": [
                        {
                            "name": "A",
                            "fields": {
                                "type": "Named",
                                "named": [
                                    [
                                        "x",
                                        {
                                            "type": "U32"
                                        }
                                    ]
                                ]
                            }
                        },
                        {
                            "name": "B",
                            "fields": {
                                "type": "Unnamed",
                                "unnamed": [
                                    {
                                        "type": "U32"
                                    }
                                ]
                            }
                        },
                        {
                            "name": "C",
                            "fields": {
                                "type": "Unit"
                            }
                        }
                    ]
                },
                {
                    "type": "Array",
                    "element": {
                        "type": "String"
                    },
                    "length": 2
                }
            ],
            "output": {
                "type": "U32"
            }
        }
    ]
}
"#
}

blueprint! {
    struct UseImport {
        simple: Simple
    }

    impl UseImport {
        pub fn new(address: ComponentAddress) -> ComponentAddress {
            Self {
                simple: address.into(),
            }
            .instantiate()
            .globalize()
        }
    }
}

#[test]
#[should_panic] // asserts it compiles
fn test_import_from_abi() {
    let instance = Simple::from(ComponentAddress::from_str("").unwrap());

    let arg1 = Floor { x: 5, y: 12 };
    let arg2 = (1u8, 2u16);
    let arg3 = Vec::<String>::new();
    let arg4 = 5;
    let arg5 = Hello::A { x: 1 };
    let arg6 = ["a".to_owned(), "b".to_owned()];

    instance.calculate_volume(arg1, arg2, arg3, arg4, arg5, arg6);
}
