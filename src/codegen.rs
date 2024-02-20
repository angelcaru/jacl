pub mod x86_64 {
    use std::{
        fmt::{Display, Formatter},
        fs::File,
        io::Write,
    };

    use crate::{
        ir::{Instruction, Program, Value},
        parser::{BinOp, CmpOp},
    };

    pub trait Compile {
        fn compile_to_asm(&self, path: &str) -> std::io::Result<()>;
    }

    impl Compile for Program {
        fn compile_to_asm(&self, path: &str) -> std::io::Result<()> {
            let mut f = File::create(path)?;

            f.write_all(b"format ELF64 \n")?;
            f.write_all(b"section '.text' executable\n")?;
            f.write_all(b"extrn print\n")?; // TODO: unhardcode functions
            f.write_all(b"extrn print_num\n")?;
            f.write_all(b"extrn read\n")?;
            f.write_all(b"public _start\n")?;
            f.write_all(b"_start:\n")?;
            f.write_all(b"    push rbp\n")?;
            f.write_all(b"    mov rbp, rsp\n")?;
            f.write_all(format!("    sub rsp, {}\n", self.vars.len() * 8).as_bytes())?;

            for instruction in &self.code {
                compile_inst_to_asm(&mut f, instruction)?;
            }

            f.write_all(b"    leave\n")?;
            f.write_all(b"    mov rax, 60\n")?;
            f.write_all(b"    mov rdi, 0\n")?;
            f.write_all(b"    syscall\n")?;

            f.write_all(b"section '.data' writable\n")?;

            for (id, string) in self.strings.iter().enumerate() {
                f.write_all(format!("str{id}: db {}", string.len()).as_bytes())?;

                for byte in string.as_bytes() {
                    f.write_all(b",")?;
                    f.write_all(byte.to_string().as_bytes())?;
                }
                f.write_all(b"\n")?;
            }

            Ok(())
        }
    }

    const CALL_CONVENTION: [Register; 6] = [
        Register::Rdi,
        Register::Rsi,
        Register::Rdx,
        Register::Rcx,
        Register::R8,
        Register::R9
    ];

    fn compile_inst_to_asm(f: &mut File, inst: &Instruction) -> std::io::Result<()> {
        use Instruction::*;
        match inst {
            FuncCall(name, args) => {
                // TODO: add support for more than six args

                for (arg, reg) in args.iter().zip(CALL_CONVENTION) {
                    move_value_into_register(f, arg, reg)?;
                }

                f.write_all(format!("    call {name}\n").as_bytes())?;
            }
            VarAssign(id, value) => {
                move_value_into_register(f, value, Register::Rdi)?;
                f.write_all(format!("    mov [rbp-{}], rdi\n", id * 8).as_bytes())?;
            }
            Label(id) => {
                f.write_all(format!("label{id}:\n").as_bytes())?;
            }
            JmpIfZero(cond, label_id) => {
                move_value_into_register(f, cond, Register::Rax)?;
                f.write_all(b"    test rax, rax\n")?;
                f.write_all(format!("    jz label{label_id}\n").as_bytes())?;
            }
            Jmp(label_id) => {
                f.write_all(format!("    jmp label{label_id}\n").as_bytes())?;
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    #[derive(Clone, Copy)]
    enum Register {
        Rax,
        Rbx,
        Rcx,
        Rdx,
        Rbp,
        Rsp,
        Rsi,
        Rdi,
        R8,
        R9,
        R10,
        R11,
        R12,
        R13,
        R14,
        R15,
    }

    impl Display for Register {
        fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
            fmt.write_str(match self {
                Register::Rax => "rax",
                Register::Rbx => "rbx",
                Register::Rcx => "rcx",
                Register::Rdx => "rdx",
                Register::Rbp => "rbp",
                Register::Rsp => "rsp",
                Register::Rsi => "rsi",
                Register::Rdi => "rdi",
                Register::R8 => "r8",
                Register::R9 => "r9",
                Register::R10 => "r10",
                Register::R11 => "r11",
                Register::R12 => "r12",
                Register::R13 => "r13",
                Register::R14 => "r14",
                Register::R15 => "r15",
            })
        }
    }

    fn move_value_into_register(f: &mut File, value: &Value, reg: Register) -> std::io::Result<()> {
        match value {
            Value::Void => {}
            &Value::String(id) => {
                f.write_all(format!("    mov {}, str{id}\n", reg).as_bytes())?;
            }
            &Value::FromVar(id) => {
                f.write_all(format!("    mov {}, [rbp-{}]\n", reg, id * 8).as_bytes())?;
            }
            &Value::Int(int) => {
                f.write_all(format!("    mov {}, {int}\n", reg).as_bytes())?;
            }
            Value::BinOp(op, a, b) => {
                move_value_into_register(f, a, Register::Rax)?;
                move_value_into_register(f, b, Register::Rbx)?;

                if let BinOp::Div = op {
                    f.write_all(b"    push rdx\n")?;
                    f.write_all(b"    xor rdx, rdx\n")?;
                    f.write_all(b"    div rbx\n")?;
                    f.write_all(b"    pop rdx\n")?;
                } else {
                    f.write_all(match op {
                        BinOp::Plus => b"    add rax, rbx\n",
                        BinOp::Minus => b"    sub rax, rbx\n",
                        BinOp::Mult => b"    mul rbx\n",
                        BinOp::Div => panic!("unreachable"),
                    })?;
                }

                f.write_all(format!("    mov {}, rax\n", reg).as_bytes())?;
            }
            Value::CmpOp(op, a, b) => {
                move_value_into_register(f, a, Register::Rax)?;
                move_value_into_register(f, b, Register::Rbx)?;

                f.write_all(b"    cmp rax, rbx\n")?;
                f.write_all(b"    mov rax, 0\n")?;
                f.write_all(b"    mov rbx, 1\n")?;
                f.write_all(match op {
                    CmpOp::Less => b"    cmovb rax, rbx\n",
                    CmpOp::Equal => b"    cmove rax, rbx\n",
                    CmpOp::Greater => b"    cmova rax, rbx\n",
                    CmpOp::LtEq => b"    cmovbe rax, rbx\n",
                    CmpOp::GtEq => b"    cmovae rax, rbx\n",
                })?;

                f.write_all(format!("    mov {}, rax\n", reg).as_bytes())?;
            }
        }
        Ok(())
    }
}
