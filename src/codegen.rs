use crate::semantic::*;
use std::fmt::Write;

pub struct Codegen;

// 공통 ENTRY POINT = main
const ENTRY: &str = "main";

// =====================================================
// 아키텍처 자동 감지
// =====================================================
fn detect_arch() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x86_64"
    }
}

impl Codegen {
    // =====================================================
    // generate() → 아키텍처 분기
    // =====================================================
    pub fn generate(&self, ir: &IRProgram) -> String {
        let arch = detect_arch();

        if arch == "arm64" {
            self.generate_arm64(ir)
        } else {
            self.generate_x86_64(ir)
        }
    }

    // =====================================================
    // X86_64 BACKEND (네 기존 코드 그대로)
    // =====================================================
    pub fn generate_x86_64(&self, ir: &IRProgram) -> String {
        let mut out = String::new();

        // DATA
        writeln!(&mut out, "section .data").unwrap();
        writeln!(&mut out, "fmt_str: db \"%s\", 0").unwrap();

        let mut strs = Vec::new();
        for f in &ir.funcs {
            for stmt in &f.body {
                self.collect_str(stmt, &mut strs);
            }
        }

        for (i, s) in strs.iter().enumerate() {
            writeln!(&mut out, "str_{}: db \"{}\", 0", i, s).unwrap();
        }

        // TEXT
        writeln!(&mut out, "section .text").unwrap();
        writeln!(&mut out, "global {}", ENTRY).unwrap();

        #[cfg(target_os = "macos")]
        writeln!(&mut out, "extern _printf").unwrap();

        #[cfg(not(target_os = "macos"))]
        writeln!(&mut out, "extern printf").unwrap();

        for f in &ir.funcs {
            writeln!(&mut out, "global {}_func", f.name).unwrap();
            writeln!(&mut out, "global {}_func_end", f.name).unwrap();
        }

        for f in &ir.funcs {
            self.gen_function_x86(&mut out, f, &strs);
        }

        // ENTRY main()
        writeln!(&mut out, "{}:", ENTRY).unwrap();
        writeln!(&mut out, "    call main_func").unwrap();
        writeln!(&mut out, "    mov eax, 0").unwrap();
        writeln!(&mut out, "    ret").unwrap();

        out
    }

    fn gen_function_x86(&self, out: &mut String, f: &IRFunction, strs: &Vec<String>) {
        writeln!(out, "{}_func:", f.name).unwrap();
        for stmt in &f.body {
            self.gen_stmt_x86(out, stmt, strs);
        }
        writeln!(out, "{}_func_end:", f.name).unwrap();
        writeln!(out, "    ret").unwrap();
    }

    fn gen_stmt_x86(&self, out: &mut String, stmt: &IR, strs: &Vec<String>) {
        match stmt {
            IR::Return(expr) => {
                self.gen_expr_x86(out, expr, strs);
                writeln!(out, "    ret").unwrap();
            }

            IR::Println(expr) => {
                self.gen_print_x86(out, expr, strs);
            }

            IR::StoreVar(_, expr) => {
                self.gen_expr_x86(out, expr, strs);
            }

            _ => {}
        }
    }

    fn gen_expr_x86(&self, out: &mut String, expr: &IRExpr, strs: &Vec<String>) {
        match expr {
            IRExpr::Int(n) => writeln!(out, "    mov rax, {}", n).unwrap(),

            IRExpr::Str(s) => {
                let idx = strs.iter().position(|x| x == s).unwrap();
                writeln!(out, "    lea rax, [rel str_{}]", idx).unwrap();
            }

            _ => {}
        }
    }

    fn gen_print_x86(&self, out: &mut String, expr: &IRExpr, strs: &Vec<String>) {
        let idx = if let IRExpr::Str(s) = expr {
            strs.iter().position(|x| x == s).unwrap()
        } else {
            panic!("println only supports string literal");
        };

        #[cfg(target_os = "macos")]
        {
            writeln!(out, "    lea rdi, [rel fmt_str]").unwrap();
            writeln!(out, "    lea rsi, [rel str_{}]", idx).unwrap();
            writeln!(out, "    sub rsp, 32").unwrap();
            writeln!(out, "    call _printf").unwrap();
            writeln!(out, "    add rsp, 32").unwrap();
            return;
        }

        #[cfg(not(target_os = "macos"))]
        {
            writeln!(out, "    lea rcx, [rel fmt_str]").unwrap();
            writeln!(out, "    lea rdx, [rel str_{}]", idx).unwrap();
            writeln!(out, "    sub rsp, 32").unwrap();
            writeln!(out, "    call printf").unwrap();
            writeln!(out, "    add rsp, 32").unwrap();
        }
    }

    // X86 string collector
    fn collect_str(&self, stmt: &IR, out: &mut Vec<String>) {
        if let IR::Println(IRExpr::Str(s)) = stmt {
            out.push(s.clone());
        }
    }

    // =====================================================
    // ARM64 BACKEND (완전한 printf 기반)
    // macOS ARM64 + Linux ARM64 둘 다 동작
    // =====================================================
    pub fn generate_arm64(&self, ir: &IRProgram) -> String {
        let mut out = String::new();

        // DATA
        out.push_str(".data\n");
        out.push_str("fmt_str:\n    .asciz \"%s\"\n");

        let mut strs = Vec::new();
        for f in &ir.funcs {
            for stmt in &f.body {
                if let IR::Println(IRExpr::Str(s)) = stmt {
                    strs.push(s.clone());
                }
            }
        }

        for (i, s) in strs.iter().enumerate() {
            writeln!(out, "str_{}:\n    .asciz \"{}\"", i, s).unwrap();
        }

        // TEXT
        out.push_str(".text\n");
        out.push_str(".global _main\n");

        // ENTRY main()
        out.push_str("_main:\n");
        out.push_str("    stp x29, x30, [sp, -16]!\n");
        out.push_str("    mov x29, sp\n");
        out.push_str("    bl main_func\n");
        out.push_str("    mov w0, 0\n");
        out.push_str("    ldp x29, x30, [sp], 16\n");
        out.push_str("    ret\n\n");

        // FUNCTIONS
        for f in &ir.funcs {
            writeln!(out, "{}_func:", f.name).unwrap();
            for stmt in &f.body {
                self.gen_stmt_arm64(&mut out, stmt, &strs);
            }
            writeln!(out, "{}_func_end:", f.name).unwrap();
            out.push_str("    ret\n\n");
        }

        out
    }

    fn gen_stmt_arm64(&self, out: &mut String, stmt: &IR, strs: &Vec<String>) {
        match stmt {
            IR::Return(expr) => {
                self.gen_expr_arm64(out, expr, strs);
                out.push_str("    ret\n");
            }
            IR::Println(expr) => {
                self.gen_print_arm64(out, expr, strs);
            }
            _ => {}
        }
    }

    fn gen_expr_arm64(&self, out: &mut String, expr: &IRExpr, strs: &Vec<String>) {
        if let IRExpr::Str(s) = expr {
            let idx = strs.iter().position(|x| x == s).unwrap();
            writeln!(out, "    adrp x0, str_{}@PAGE", idx).unwrap();
            writeln!(out, "    add  x0, x0, str_{}@PAGEOFF", idx).unwrap();
        }
    }

    fn gen_print_arm64(&self, out: &mut String, expr: &IRExpr, strs: &Vec<String>) {
        let idx = if let IRExpr::Str(s) = expr {
            strs.iter().position(|x| x == s).unwrap()
        } else {
            panic!("println only supports string literal");
        };

        // x0 = fmt_str
        out.push_str("    adrp x0, fmt_str@PAGE\n");
        out.push_str("    add  x0, x0, fmt_str@PAGEOFF\n");

        // x1 = str_x
        writeln!(out, "    adrp x1, str_{}@PAGE", idx).unwrap();
        writeln!(out, "    add  x1, x1, str_{}@PAGEOFF", idx).unwrap();

        // printf
        out.push_str("    bl _printf\n");
    }
}
