# ============================
# Minimal return
# ============================
func test_minimal() {
    return;
}

# ============================
# Arithmetic test
# ============================
func test_arithmetic() {
    let a = 10;
    let b = 20;
    let c = a + b;
    print(c);
}

# ============================
# Semantic / logic test
# ============================
func test_semantic() {
    let x = 1;
    let y = x * 4 + 2;

    if y > 4 {
        print("ok");
    } else {
        print("no");
    }
}

# ============================
# entrypoint
# ============================
func main() {
    test_minimal();
    test_arithmetic();
    test_semantic();
}
