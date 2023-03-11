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

mod dev_att;
use matter::core::{self, CommissioningData};
use matter::data_model::cluster_basic_information::BasicInfoConfig;
use matter::data_model::cluster_on_off::{Commands, OnOffCluster};
use matter::data_model::device_types::device_type_add_on_off_light;
use matter::secure_channel::spake2p::VerifierData;

fn main() {
    env_logger::init();
    let comm_data = CommissioningData {
        // TODO: Hard-coded for now
        verifier: VerifierData::new_with_pw(123456),
        discriminator: 250,
    };

    // vid/pid should match those in the DAC
    let dev_info = BasicInfoConfig {
        vid: 0xFFF1,
        pid: 0x8000,
        hw_ver: 2,
        sw_ver: 1,
        sw_ver_str: "1".to_string(),
        serial_no: "aabbccdd".to_string(),
        device_name: "OnOff Light".to_string(),
    };
    let dev_att = Box::new(dev_att::HardCodedDevAtt::new());

    let mut matter = core::Matter::new(dev_info, dev_att, comm_data).unwrap();
    let dm = matter.get_data_model();
    {
        let mut node = dm.node.write().unwrap();
        let endpoint = device_type_add_on_off_light(&mut node).unwrap();
        println!("Added OnOff Light Device type at endpoint id: {}", endpoint);
        println!("Data Model now is: {}", node);

        let mut on_off_cluster = OnOffCluster::new().unwrap();
        let on_callback = Box::new(|| log::info!("Comamnd [On] handled with callback."));
        let off_callback = Box::new(|| log::info!("Comamnd [Off] handled with callback."));
        on_off_cluster.add_callback(Commands::On, on_callback);
        on_off_cluster.add_callback(Commands::Off, off_callback);

        node.add_cluster(endpoint, on_off_cluster).unwrap();
        println!("Added On/Off type at endpoint id: {}", endpoint)
    }

    matter.start_daemon().unwrap();
}
