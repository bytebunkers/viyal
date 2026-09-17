class Main {
    void run() {
        var myMap = {"name": "Viyal", "version": "1.0"};
        print(myMap["name"]);
        print(myMap["version"]);
        
        myMap["version"] = "1.1";
        print(myMap["version"]);
        
        myMap["new_key"] = "hello";
        print(myMap["new_key"]);
    }
}
