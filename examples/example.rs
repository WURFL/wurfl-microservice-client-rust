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
        println!("WURFL Microservice client created successfully. Rust client API version: {}", client.get_api_version());
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
    println!("Print the first {} Brand of {} found", limit, device_makes.len());

    for _i in 0..limit {
        println!("{}", device_makes.get(_i).unwrap());
    }

}