use crate::parser::{BinOp, CmpOp, Node};

pub struct Program {
    pub strings: Vec<String>,
    pub code: Vec<Instruction>,
    pub vars: Vec<String>,
    pub label_count: usize,
    backpatch_stack: Vec<usize>,
}

#[derive(Debug)]
pub enum Value {
    Void,
    String(usize),
    FromVar(usize),
    Int(usize),
    BinOp(BinOp, Box<Value>, Box<Value>),
    CmpOp(CmpOp, Box<Value>, Box<Value>),
}

#[derive(Debug)]
pub enum Instruction {
    FuncCall(String, Vec<Value>),
    VarAssign(usize, Value),
    Label(usize),
    JmpIfZero(Value, usize),
    Jmp(usize),
}

impl Instruction {
    fn backpatch(&mut self, label_id: usize) {
        match self {
            Self::JmpIfZero(_, i) => {
                *i = label_id;
            }
            Self::Jmp(i) => {
                *i = label_id;
            }
            _ => panic!("backpatch() called on non-jump instruction: {:?}", self)
        }
    }
}

impl Program {
    pub fn disassemble(&self) {
        println!("BEGIN DISASSEMBLY");
        println!("Strings: ");
        for (i, string) in self.strings.iter().enumerate() {
            println!("  {i}: {string:?}");
        }

        println!("Variables: ");
        for (i, name) in self.vars.iter().enumerate() {
            println!("  {i}: {name}");
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
        let mut prog = Program {
            strings,
            code,
            vars,
            label_count: 0,
            backpatch_stack: Vec::new()
        };

        prog.visit(node);

        prog
    }

    fn visit(&mut self, node: &Node) -> Value {
        match node {
            Node::FuncCall(name, args) => {
                let args: Vec<_> = args.iter().map(|arg| self.visit(arg)).collect();

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
                self.code
                    .push(Instruction::VarAssign(self.vars.len(), value));
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
            Node::VarAssign(name, node) => {
                if let Some(idx) = self.vars.iter().position(|x| x == name) {
                    let value = self.visit(node);
                    self.code.push(Instruction::VarAssign(idx, value));
                    Value::Void
                } else {
                    panic!("Undeclared variable: {}", name);
                }
            }
            &Node::Int(int) => Value::Int(int),
            Node::BinOp(op, a, b) => {
                let a = self.visit(a);
                let b = self.visit(b);

                Value::BinOp(*op, Box::new(a), Box::new(b))
            }
            Node::CmpOp(op, a, b) => {
                let a = self.visit(a);
                let b = self.visit(b);

                Value::CmpOp(*op, Box::new(a), Box::new(b))
            }
            Node::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond = self.visit(cond);

                self.backpatch_stack.push(self.code.len());
                self.code.push(Instruction::JmpIfZero(cond, 0));

                self.visit(then_branch);
                let i = self.backpatch_stack.pop().unwrap();
                if else_branch.is_some() {
                    self.backpatch_stack.push(self.code.len());
                    self.code.push(Instruction::Jmp(0));
                }
                let label = self.add_label();
                self.code[i].backpatch(label);

                if let Some(else_branch) = else_branch {
                    self.visit(else_branch);
                    
                    self.backpatch();
                }

                Value::Void
            }
            Node::While { cond, body } => {
                let start_label = self.add_label();

                let cond = self.visit(cond);
                self.backpatch_stack.push(self.code.len());
                self.code.push(Instruction::JmpIfZero(cond, 0));

                self.visit(body);

                self.code.push(Instruction::Jmp(start_label));

                self.backpatch();

                Value::Void
            }
            Node::Nop => Value::Void,
        }
    }

    fn backpatch(&mut self) {
        let i = self.backpatch_stack.pop().unwrap();
        let label = self.add_label();
        self.code[i].backpatch(label);
    }

    fn add_label(&mut self) -> usize {
        self.code.push(Instruction::Label(self.label_count));
        self.label_count += 1;
        self.label_count - 1
    }
}
