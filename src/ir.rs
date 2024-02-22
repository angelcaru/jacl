use std::{collections::HashMap, fmt::Display};

use crate::{
    loc::Loc,
    parser::{BinOp, CmpOp, Node},
};

pub struct Program {
    pub strings: Vec<String>,
    pub scopes: HashMap<String, Vec<String>>,
    pub label_count: usize,
    pub fn_bodies: HashMap<String, Vec<Instruction>>,
    pub bufs: Vec<usize>,
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
    Buf(usize),
    PtrAccess(Box<Value>),
    VarAddr(usize),
    FuncCall(String, Vec<Value>),
}

#[derive(Debug)]
pub enum Instruction {
    VarAssign(usize, Value),
    Label(usize),
    JmpIfZero(Value, usize),
    Jmp(usize),
    Prologue(usize),
    Return(Value),
    Exit(u8),
    PtrAssign(Value, Value),
    EvalValue(Value),
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
            _ => panic!("backpatch() called on non-jump instruction: {:?}", self),
        }
    }
}

#[derive(Debug)]
pub struct IRError(Loc, String);

impl Display for IRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let IRError(loc, msg) = self;
        f.write_fmt(format_args!("{}: {}", loc, msg))
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
        for (fun, vars) in &self.scopes {
            println!("fun {}():", fun);
            for (i, name) in vars.iter().enumerate() {
                println!("  {i}: {name}");
            }
        }

        println!("Code: ");
        for (fun, code) in &self.fn_bodies {
            println!("fun {}():", fun);
            for inst in code {
                println!("  {inst:?}");
            }
        }

        println!("END DISASSEMBLY");
    }

    pub fn from_ast(node: &Node) -> Result<Program, IRError> {
        let strings = Vec::new();
        let mut code = Vec::new();
        let mut vars = Vec::new();
        let mut prog = Program {
            strings,
            fn_bodies: HashMap::new(),
            scopes: HashMap::new(),
            label_count: 0,
            bufs: Vec::new(),
            backpatch_stack: Vec::new(),
        };

        code.push(Instruction::Prologue(0));
        prog.visit(node, &mut vars, &mut code)?;
        code.push(Instruction::Exit(0));

        prog.fn_bodies.insert("_start".into(), code);
        prog.scopes.insert("_start".into(), vars);

        Ok(prog)
    }

    fn visit(
        &mut self,
        node: &Node,
        scope: &mut Vec<String>,
        code: &mut Vec<Instruction>,
    ) -> Result<Value, IRError> {
        Ok(match node {
            Node::FuncCall(_, name, args) => {
                let args: Result<Vec<_>, _> = args
                    .iter()
                    .map(|arg| self.visit(arg, scope, code))
                    .collect();

                Value::FuncCall(name.clone(), args?)
            }
            Node::StrLit(_, string) => {
                if let Some(idx) = self.strings.iter().position(|x| x == string) {
                    Value::String(idx)
                } else {
                    self.strings.push(string.clone());
                    Value::String(self.strings.len() - 1)
                }
            }
            Node::Block(_, nodes) => {
                for node in nodes {
                    let val = self.visit(node, scope, code)?;
                    code.push(Instruction::EvalValue(val));
                }
                Value::Void
            }
            Node::VarDecl(loc, name, node) => {
                if scope.contains(name) {
                    return Err(IRError(
                        loc.clone(),
                        format!("Already declared variable: {}", name),
                    ));
                }
                let value = self.visit(node, scope, code)?;
                code.push(Instruction::VarAssign(scope.len(), value));
                scope.push(name.clone());
                Value::Void
            }
            Node::VarAccess(loc, name) => {
                if let Some(idx) = scope.iter().position(|x| x == name) {
                    Value::FromVar(idx)
                } else {
                    return Err(IRError(
                        loc.clone(),
                        format!("Undeclared variable: {}", name),
                    ));
                }
            }
            Node::VarAssign(loc, name, node) => {
                if let Some(idx) = scope.iter().position(|x| x == name) {
                    let value = self.visit(node, scope, code)?;
                    code.push(Instruction::VarAssign(idx, value));
                    Value::Void
                } else {
                    return Err(IRError(
                        loc.clone(),
                        format!("Undeclared variable: {}", name),
                    ));
                }
            }
            &Node::Int(_, int) => Value::Int(int),
            Node::BinOp(_, op, a, b) => {
                let a = self.visit(a, scope, code)?;
                let b = self.visit(b, scope, code)?;

                Value::BinOp(*op, Box::new(a), Box::new(b))
            }
            Node::CmpOp(_, op, a, b) => {
                let a = self.visit(a, scope, code)?;
                let b = self.visit(b, scope, code)?;

                Value::CmpOp(*op, Box::new(a), Box::new(b))
            }
            Node::If {
                loc: _,
                cond,
                then_branch,
                else_branch,
            } => {
                let cond = self.visit(cond, scope, code)?;

                self.backpatch_stack.push(code.len());
                code.push(Instruction::JmpIfZero(cond, 0));

                self.visit(then_branch, scope, code)?;
                let i = self.backpatch_stack.pop().unwrap();
                if else_branch.is_some() {
                    self.backpatch_stack.push(code.len());
                    code.push(Instruction::Jmp(0));
                }
                let label = self.add_label(code);
                code[i].backpatch(label);

                if let Some(else_branch) = else_branch {
                    self.visit(else_branch, scope, code)?;

                    self.backpatch(code);
                }

                Value::Void
            }
            Node::While { loc: _, cond, body } => {
                let start_label = self.add_label(code);

                let cond = self.visit(cond, scope, code)?;
                self.backpatch_stack.push(code.len());
                code.push(Instruction::JmpIfZero(cond, 0));

                self.visit(body, scope, code)?;

                code.push(Instruction::Jmp(start_label));

                self.backpatch(code);

                Value::Void
            }
            Node::FuncDef {
                loc: _,
                name,
                args,
                body,
            } => {
                let mut body_code = Vec::new();
                let mut body_vars = Vec::new();

                body_code.push(Instruction::Prologue(args.len()));
                for arg in args {
                    body_vars.push(arg.clone());
                }

                self.visit(body, &mut body_vars, &mut body_code)?;
                body_code.push(Instruction::Return(Value::Void));

                self.fn_bodies.insert(name.clone(), body_code);
                self.scopes.insert(name.clone(), body_vars);
                Value::Void
            }
            &Node::Buf(_, size) => {
                self.bufs.push(size);
                Value::Buf(self.bufs.len() - 1)
            }
            Node::PtrAssign(_, ptr, val) => {
                let ptr = self.visit(ptr, scope, code)?;
                let val = self.visit(val, scope, code)?;

                code.push(Instruction::PtrAssign(ptr, val));

                Value::Void
            }
            Node::PtrAccess(_, ptr) => {
                let ptr = self.visit(ptr, scope, code)?;
                Value::PtrAccess(Box::new(ptr))
            },
            Node::VarAddr(loc, name) => {
                if let Some(id) = scope.iter().position(|x| x == name) {
                    Value::VarAddr(id)
                } else {
                    return Err(IRError(loc.clone(), format!("Undeclared variable: {name}")));
                }
            }
            Node::Return(_, val) => {
                let val = self.visit(val, scope, code)?;
                code.push(Instruction::Return(val));
                Value::Void
            }
            Node::Nop(_) => Value::Void,
        })
    }

    fn backpatch(&mut self, code: &mut Vec<Instruction>) {
        // TODO/TOFIGUREOUT: Maybe this isn't the best idea
        let i = self.backpatch_stack.pop().unwrap();
        let label = self.add_label(code);
        code[i].backpatch(label);
    }

    fn add_label(&mut self, code: &mut Vec<Instruction>) -> usize {
        code.push(Instruction::Label(self.label_count));
        self.label_count += 1;
        self.label_count - 1
    }
}
