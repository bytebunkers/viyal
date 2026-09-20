class Main {
    void run() {
        print(math.abs(0 - 5));
        
        fs.writeText("test_output.txt", "hello stdlib!");
        print(fs.readText("test_output.txt"));
        
        print(os.getenv("PATH"));
        
        print(time.now());
        
        // Let's test json
        var parsed = json.parse(fs.readText("test.json"));
        print(parsed);
        
        var stringified = json.stringify(parsed);
        print(stringified);
    }
}
