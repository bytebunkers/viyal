function fib(n) {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}

const start = performance.now();
const res = fib(35);
const end = performance.now();
console.log(`Node.js: ${res} in ${(end - start) / 1000}s`);
