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

    // More to come...

}