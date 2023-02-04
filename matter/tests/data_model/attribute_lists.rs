/*
 *
 *    Copyright (c) 2020-2022 Project CHIP Authors
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use matter::{
    data_model::{core::DataModel, objects::EncodeValue},
    interaction_model::{
        core::{IMStatusCode, OpCode},
        messages::GenericPath,
        messages::{
            ib::{AttrData, AttrPath, AttrStatus},
            msg::{WriteReq, WriteResp},
        },
    },
    tlv::{self, FromTLV, Nullable},
};

use crate::common::{
    echo_cluster::{self, TestChecker},
    im_engine::im_engine,
};

// Helper for handling Write Attribute sequences
fn handle_write_reqs(input: &[AttrData], expected: &[AttrStatus]) -> DataModel {
    let mut out_buf = [0u8; 400];
    let write_req = WriteReq::new(false, input);

    let (dm, _, out_buf) = im_engine(OpCode::WriteRequest, &write_req, &mut out_buf);
    tlv::print_tlv_list(out_buf);
    let root = tlv::get_root_node_struct(out_buf).unwrap();
    let resp = WriteResp::from_tlv(&root).unwrap();
    assert_eq!(resp.write_responses, expected);
    dm
}

#[test]
/// This tests all the attribute list operations
/// add item, edit item, delete item, overwrite list, delete list
fn attr_list_ops() {
    let val0: u16 = 10;
    let val1: u16 = 15;
    let tc_handle = TestChecker::get().unwrap();

    let _ = env_logger::try_init();

    let delete_item = EncodeValue::Closure(&|tag, t| {
        let _ = t.null(tag);
    });
    let delete_all = EncodeValue::Closure(&|tag, t| {
        let _ = t.start_array(tag);
        let _ = t.end_container();
    });

    let att_data = GenericPath::new(
        Some(0),
        Some(echo_cluster::ID),
        Some(echo_cluster::Attributes::AttWriteList as u32),
    );
    let mut att_path = AttrPath::new(&att_data);

    // Test 1: Add Operation - add val0
    let input = &[AttrData::new(None, att_path, EncodeValue::Value(&val0))];
    let expected = &[AttrStatus::new(&att_data, IMStatusCode::Success, 0)];
    let _ = handle_write_reqs(input, expected);

    {
        let tc = tc_handle.lock().unwrap();
        assert_eq!([Some(val0), None, None, None, None], tc.write_list);
    }

    // Test 2: Another Add Operation - add val1
    let input = &[AttrData::new(None, att_path, EncodeValue::Value(&val1))];
    let expected = &[AttrStatus::new(&att_data, IMStatusCode::Success, 0)];
    let _ = handle_write_reqs(input, expected);

    {
        let tc = tc_handle.lock().unwrap();
        assert_eq!([Some(val0), Some(val1), None, None, None], tc.write_list);
    }

    // Test 3: Edit Operation - edit val1 to val0
    att_path.list_index = Some(Nullable::NotNull(1));
    let input = &[AttrData::new(None, att_path, EncodeValue::Value(&val0))];
    let expected = &[AttrStatus::new(&att_data, IMStatusCode::Success, 0)];
    let _ = handle_write_reqs(input, expected);

    {
        let tc = tc_handle.lock().unwrap();
        assert_eq!([Some(val0), Some(val0), None, None, None], tc.write_list);
    }

    // Test 4: Delete Operation - delete index 0
    att_path.list_index = Some(Nullable::NotNull(0));
    let input = &[AttrData::new(None, att_path, delete_item)];
    let expected = &[AttrStatus::new(&att_data, IMStatusCode::Success, 0)];
    let _ = handle_write_reqs(input, expected);

    {
        let tc = tc_handle.lock().unwrap();
        assert_eq!([None, Some(val0), None, None, None], tc.write_list);
    }

    // Test 5: Overwrite Operation - overwrite first 2 entries
    let overwrite_val: [u32; 2] = [20, 21];
    att_path.list_index = None;
    let input = &[AttrData::new(
        None,
        att_path,
        EncodeValue::Value(&overwrite_val),
    )];
    let expected = &[AttrStatus::new(&att_data, IMStatusCode::Success, 0)];
    let _ = handle_write_reqs(input, expected);

    {
        let tc = tc_handle.lock().unwrap();
        assert_eq!([Some(20), Some(21), None, None, None], tc.write_list);
    }

    // Test 6: Overwrite Operation - delete whole list
    att_path.list_index = None;
    let input = &[AttrData::new(None, att_path, delete_all)];
    let expected = &[AttrStatus::new(&att_data, IMStatusCode::Success, 0)];
    let _ = handle_write_reqs(input, expected);

    {
        let tc = tc_handle.lock().unwrap();
        assert_eq!([None, None, None, None, None], tc.write_list);
    }
}
