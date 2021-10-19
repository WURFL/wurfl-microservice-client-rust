# WURFL Microservice client for Rust
==============

WURFL Microservice (by ScientiaMobile, Inc.) is a mobile device detection service that can quickly and accurately detect over 500 capabilities of visiting devices. It can differentiate between portable mobile devices, desktop devices, SmartTVs and any other types of devices that have a web browser.

This is the Rust Client API for accessing the WURFL Microservice. The API is released under Open-Source and can be integrated with other open-source or proprietary code. In order to operate, it requires access to a running instance of the WURFL Microservice product, such as:

- WURFL Microservice for Docker: https://www.scientiamobile.com/products/wurfl-microservice-docker-detect-device/

- WURFL Microservice for AWS: https://www.scientiamobile.com/products/wurfl-device-detection-microservice-aws/

- WURFL Microservice for Azure: https://www.scientiamobile.com/products/wurfl-microservice-for-azure/

- WURFL Microservice for Google Cloud Platform: https://www.scientiamobile.com/products/wurfl-microservice-for-gcp/

Compiling the WURFL microservice client requires Rust 1.48 or above.

The Example project contains an example of client api usage for a console application :

```rust
use std::collections::HashMap;
use wmclient::WmClient;

fn main() {
    // Let's create the WURFL microservice client by setting the connection data of out WURFL Microservice server
    let client_res = WmClient::new("http", "localhost", "8080", "");
    // Client is mutable because because some its internal can be modified depending on its interaction with the user or
    // the server.
    let mut client: WmClient;
    if client_res.is_ok(){
        client = client_res.unwrap();
        println!("-----------------------------------------------------------------------------------");
        println!("WURFL Microservice client created successfully. Rust client API version: {}", client.get_api_version());
        println!("-----------------------------------------------------------------------------------");
    } else {
        println!("Unable to create WURFL Microservice client: {}", client_res.err().unwrap().to_string());
        return;
    }
    // Let's add the caching layer to the client
    client.set_cache_size(10000);
    // Let's gather some server info.
    let info_res = client.get_info();
    if info_res.is_err(){
        println!("Unable to get server info. Exiting.");
        return;
    }
    let info = info_res.unwrap();
    println!("WURFL Microservice information:");
    println!("Server version: {}", info.wm_version);
    println!("WURFL API version: {}", info.wurfl_api_version);
    println!("WURFL file info: {}", info.wurfl_info);

    // set the capabilities we want to receive from WM server
    // Static capabilities
    let static_caps = vec! {"model_name brand_name"};
    client.set_requested_static_capabilities(Some(static_caps));
    // Virtual capabilities
    let virtual_caps = vec! {"is_smartphone form_factor"};
    client.set_requested_virtual_capabilities(Some(virtual_caps));

    // use this headers to perform a device detection.
    let mut headers = HashMap::new();
    headers.insert("Content-Type", "application/json");
    headers.insert("Accept", "text/html, application/xml;q=0.9, application/xhtml+xml, image/png, image/webp, image/jpeg, image/gif, image/x-xbitmap, */*;q=0.1");
    headers.insert("Accept-Encoding", "gzip, deflate");
    headers.insert("Accept-Language", "en");
    headers.insert("Device-Stock-Ua", "Mozilla/5.0 (Linux; Android 8.1.0; SM-J610G Build/M1AJQ; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/69.0.3497.100 Mobile Safari/537.36");
    headers.insert("Forwarded", "for=\"110.54.224.195:36350\"");
    headers.insert("Referer", "https://www.cram.com/flashcards/labor-and-delivery-questions-889210");
    headers.insert("User-Agent", "Opera/9.80 (Android; Opera Mini/51.0.2254/184.121; U; en) Presto/2.12.423 Version/12.16");
    headers.insert("X-Clacks-Overhead", "GNU ph");
    headers.insert("X-Forwarded-For", "110.54.224.195, 82.145.210.235");
    headers.insert("X-Operamini-Features", "advanced, camera, download, file_system, folding, httpping, pingback, routing, touch, viewport");
    headers.insert("X-Operamini-Phone", "Android #");
    headers.insert("X-Operamini-Phone-Ua", "Mozilla/5.0 (Linux; Android 8.1.0; SM-J610G Build/M1AJQ; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/69.0.3497.100 Mobile Safari/537.36");

    let device_res = client.lookup_headers(headers);
    if device_res.is_err(){
        println!("Unable to detect device from the given HTTP headers: {}", device_res.err().unwrap().to_string());
        return;
    }
    // No error, let's get the device data
    let device = device_res.unwrap();
    let wurfl_id_opt = device.capabilities.get("wurfl_id");
    if wurfl_id_opt.is_some() {
        println!("-----------------------------------------------------------------------------------");
        println!("Sample device detection using sample headers");
        println!("WURFL device ID : {}", wurfl_id_opt.unwrap());
    }
    // If you are sure the capability you're querying exists and is in your required set, just unwrap the capability option
    println!("This device is a : {} {}", device.capabilities.get("brand_name").unwrap(), device.capabilities.get("model_name").unwrap());

    // check if device is a smartphone (a virtual capability)
    if device.capabilities.get("is_smartphone").unwrap() == "true" {
        println!("This is a smartphone")
    }
    println!("This device form_factor is: {}", device.capabilities.get("form_factor").unwrap());

    // Get all the device manufacturers, and print the first twenty
    let makes_res = client.get_all_device_makes();
    if makes_res.is_err() {
        let err_mk = makes_res.as_ref().err().unwrap();
        println!("Error getting device makes data {}", err_mk.to_string());
    }


    let mut device_makes = makes_res.unwrap();
    device_makes.sort();
    let limit = 20;
    println!("-----------------------------------------------------------------------------------");
    println!("Print the first {} brand_names of {} found", limit, device_makes.len());
    println!("-----------------------------------------------------------------------------------");

    for _i in 0..limit {
        println!("{}", device_makes.get(_i).unwrap());
    }

    // Now call the WM server to get all device model and marketing names produced by Apple
    let model_marketing_names_opt = client.get_all_devices_for_make("Apple".to_string());
    if model_marketing_names_opt.is_err(){
        let err_mmkt = model_marketing_names_opt.as_ref().err().unwrap();
        println!("Error getting device model and marketing data for Apple:  {}", err_mmkt.to_string());
    }

    let mut model_marketing_names = model_marketing_names_opt.unwrap();
    // Sort model_marketing_names structs by their model name, using natural ordering (thus Uppercase names come first, then lowercase)
    model_marketing_names.sort_by(|a,b| a.model_name.cmp(&b.model_name));
    println!("-----------------------------------------------------------------------------------");
    println!("Printing all model and marketing names for Apple brand");
    println!("-----------------------------------------------------------------------------------");
    for name in model_marketing_names{
        println!("- {} {}", name.model_name, name.marketing_name);
    }

    // Now call the WM server to get all operating system names
    println!("-----------------------------------------------------------------------------------");
    println!("Print the list of OSes");
    println!("-----------------------------------------------------------------------------------");
    let os_opt = client.get_all_oses();
    if os_opt.is_err(){
        let os_err = os_opt.as_ref().err().unwrap();
        println!("Unable to get the list of operating systems: {}", os_err.to_string());
    }
    let mut os_list = os_opt.unwrap();
    os_list.sort();
    for os in os_list {
        println!("- {}", os);
    }

    println!("-----------------------------------------------------------------------------------");
    println!("Print all version numbers for Android OS");
    println!("-----------------------------------------------------------------------------------");
    let android_ver_opt = client.get_all_versions_for_os("Android");
    if android_ver_opt.is_err(){
        let ver_err = android_ver_opt.as_ref().err().unwrap();
        println!("Unable to get versions for Android OS: {}", ver_err.to_string());
    }
    let android_versions = android_ver_opt.unwrap();
    for v in android_versions {
        println!("- {}", v);
    }
}
```

# Crates.io distribution note
`wmclient` package distributed via [crates.io](https://crates.io/search?q=wmclient) does **not** contain unit tests or examples.
If you need run the tests please clone the GitHub repo or, if you need the code of a specific release, download the zip file 
on [GitHub release page](https://github.com/WURFL/wurfl-microservice-client-rust/releases) 