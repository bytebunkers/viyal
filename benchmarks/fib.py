import time

def fib(n):
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

start = time.time()
res = fib(35)
end = time.time()
print(f"Python: {res} in {end - start:.4f}s")
