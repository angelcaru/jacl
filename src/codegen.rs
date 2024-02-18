pub mod x86_64 {
    use std::{fs::File, io::Write};

    use crate::ir::{Instruction, Program, Value};

    pub trait Compile {
        fn compile_to_asm(&self, path: &str) -> std::io::Result<()>;
    }

    impl Compile for Program {
        fn compile_to_asm(&self, path: &str) -> std::io::Result<()> {
            let mut f = File::create(path)?;

            f.write_all(b"format ELF64 \n")?;
            f.write_all(b"section '.text' executable\n")?;
            f.write_all(b"extrn print\n")?; // TODO: unhardcode functions
            f.write_all(b"extrn printNum\n")?;
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
    fn compile_inst_to_asm(f: &mut File, inst: &Instruction) -> std::io::Result<()> {
        use Instruction::*;
        match inst {
            FuncCall(name, args) => {
                assert_eq!(args.len(), 1); // Parser doesn't support it, why should we?
                let arg = &args[0];

                move_value_into_rdi(f, arg)?;

                f.write_all(format!("    call {name}\n").as_bytes())?;
            }
            VarAssign(id, value) => {
                move_value_into_rdi(f, value)?;
                f.write_all(format!("    mov [rbp-{}], rdi\n", id * 8).as_bytes())?;
            }
        }

        Ok(())
    }

    fn move_value_into_rdi(f: &mut File, value: &Value) -> std::io::Result<()> {
        match value {
            Value::Void => {}
            &Value::String(id) => {
                f.write_all(format!("    mov rdi, str{id}\n").as_bytes())?;
            }
            &Value::FromVar(id) => {
                f.write_all(format!("    mov rdi, [rbp-{}]\n", id * 8).as_bytes())?;
            }
            &Value::Int(int) => {
                f.write_all(format!("    mov rdi, {int}\n").as_bytes())?;
            }
        }
        Ok(())
    }
}
