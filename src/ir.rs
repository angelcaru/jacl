use crate::parser::Node;

pub struct Program {
    pub strings: Vec<String>,
    pub code: Vec<Instruction>,
    pub vars: Vec<String>
}

#[derive(Debug)]
pub enum Value {
    Void,
    String(usize),
    FromVar(usize),
}

#[derive(Debug)]
pub enum Instruction {
    FuncCall(String, Vec<Value>),
    VarAssign(usize, Value),
}

impl Program {
    pub fn disassemble(&self) {
        println!("BEGIN DISASSEMBLY");
        println!("Strings: ");
        for (i, string) in self.strings.iter().enumerate() {
            println!("  {i}: {string:?}");
        }

        println!("Code: ");
        for inst in &self.code {
            println!("  {inst:?}");
        }

        println!("END DISASSEMBLY");
    }

    pub fn from_ast(node: &Node) -> Program {
        let strings = Vec::new();
        let code = Vec::new();
        let vars = Vec::new();
        let mut prog = Program { strings, code, vars };

        prog.visit(node);

        prog
    }

    fn visit(&mut self, node: &Node) -> Value {
        match node {
            Node::FuncCall(name, args) => {
                let args: Vec<_> = args
                    .iter()
                    .map(|arg| self.visit(arg))
                    .collect();

                self.code.push(Instruction::FuncCall(name.clone(), args));

                Value::Void
            }
            Node::StrLit(string) => {
                if let Some(idx) = self.strings.iter().position(|x| x == string) {
                    Value::String(idx)
                } else {
                    self.strings.push(string.clone());
                    Value::String(self.strings.len() - 1)
                }
            }
            Node::Block(nodes) => {
                for node in nodes {
                    self.visit(node);
                }
                Value::Void
            }
            Node::VarDecl(name, node) => {
                if self.vars.contains(name) {
                    // TODO: proper error reporting in IR generation
                    panic!("Already declared variable: {}", name);
                }
                let value = self.visit(node);
                self.code.push(Instruction::VarAssign(self.vars.len(), value));
                self.vars.push(name.clone());
                Value::Void
            }
            Node::VarAccess(name) => {
                if let Some(idx) = self.vars.iter().position(|x| x == name) {
                    Value::FromVar(idx)
                } else {
                    panic!("Undeclared variable: {}", name);
                }
            }
        }
    }
}
