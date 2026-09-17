class Main {
    void run() {
        var arr = [10, 20, 30];
        print(arr[0]);
        print(arr[1]);
        print(arr[2]);
        
        arr[1] = 99;
        print(arr[1]);
        
        // This should cause an out of bounds error
        print(arr[5]);
    }
}
