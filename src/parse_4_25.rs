use super::lex_4_25;

#[derive(Debug)]
pub struct ParseTree {
    pub children: Vec<ParseNode>
}

#[derive(Clone, Debug)]
pub struct ParseNode {
    pub node: lex_4_25::Token,
    pub children: Vec<Box<ParseNode>>
}

impl ParseNode {
    fn add_child(&mut self, token: lex_4_25::Token) { 
        self.children.push(Box::new(ParseNode{ node: token, children: vec![] })); 
    }

    fn remove_left_child(&mut self) -> Box<ParseNode> { 
        let mut children = self.children.clone();
        self.children = children.split_off(1); 
        children.pop().unwrap()
    }

    fn merge_expressions(mut self, mut other: ParseNode) -> ParseNode {
        self.children.push(other.remove_left_child());
        other.children.push(Box::new(self));
        other.children.swap(0, 1);
        other
    }

    fn empty() -> ParseNode {
        ParseNode { node: lex_4_25::Token::NoMatch, children: vec![] }
    }
}

impl lex_4_25::Token {
    fn to_leaf(self) -> ParseNode {
        ParseNode { node: self, children: vec![] }
    }
}

/*** Top-Level ***/

pub fn parse(input: String) -> ParseTree {
    let current_node = &mut ParseNode::empty();
    let mut tree = ParseTree{ children: vec![] };
    let mut cur = &mut 0;
    let input_chars = &mut input.chars().collect::<Vec<char>>(); 
    while *cur < input_chars.len() {
        match lex_4_25::peek_token(input_chars, cur) {
            lex_4_25::Token::Pragma => {
                current_node.node = lex_4_25::Token::Pragma;
                tree.children.push(parse_pragma(input_chars, cur));
            }
            lex_4_25::Token::Import => {
                parse_import(input_chars, cur);
            }
            lex_4_25::Token::Contract => {
                current_node.node = lex_4_25::Token::Contract;
                tree.children.push(parse_contract(input_chars, cur));
            }
            lex_4_25::Token::Library => {
                current_node.node = lex_4_25::Token::Library;
                parse_contract(input_chars, cur);
            }
            lex_4_25::Token::Interface => {
                current_node.node = lex_4_25::Token::Interface;
                parse_contract(input_chars, cur);
            }
            _ => panic!("Invalid top level expression")
        }
    }
    tree
}

/*** Pragma ***/

pub fn parse_pragma(chars: &Vec<char>, cur: &mut usize) -> ParseNode { 
    let mut result = ParseNode::empty();
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::Pragma => result.node = lex_4_25::Token::Pragma,
        _ => panic!("Invalid pragma declaration")
    }
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::Identifier(name) => {
            if name != "solidity" {
                panic!("Invalid source file: Not a solidity file")
            }
            result.add_child(lex_4_25::Token::Identifier(name));
        }
        _ => panic!("2 Invalid pragma declaration")
    }
    match lex_4_25::next_token(&chars, cur) {
        lex_4_25::Token::Version(version) => {
            if version != "^0.4.25" && version != "0.4.25" {
                panic!("Invalid source file: version other than 0.4.25 specfied")
            }
            result.add_child(lex_4_25::Token::Version(version));
        }
        actual => panic!("3 Invalid pragma declaration {:?}", actual)
    }
    match lex_4_25::next_token(&chars, cur) {
        lex_4_25::Token::Semicolon => (), 
        _ => panic!("4 Invalid pragma declaration")
    }
    result
} 

/*** Import ***/

fn parse_import(chars: &Vec<char>, cur: &mut usize) -> ParseNode { ParseNode::empty() }

/*** Contract ***/

fn parse_contract(chars: &Vec<char>, cur: &mut usize) -> ParseNode {
    let mut is = false;
    let mut result = ParseNode::empty();
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::Contract => result.node = lex_4_25::Token::Contract,
        _ => panic!("Invalid contract definition")
    }
    match lex_4_25::next_token(chars, cur) {
        id@lex_4_25::Token::Identifier(..) => result.add_child(id),
        _ => panic!("Invalid contract definition")
    }
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::OpenBrace => (),
        lex_4_25::Token::Is => is = true,
        _ => panic!("Invalid contract definition")
    }
    if is {
        let mut stop = false;
        let mut is_node = lex_4_25::Token::Is.to_leaf(); 
        while !stop {
            is_node.children.push(Box::new(parse_inheritance_specifier(chars, cur)));
            if let lex_4_25::Token::Comma = lex_4_25::peek_token(chars, cur) {
                lex_4_25::next_token(chars, cur);
            } else {
                stop = true;
            }
        }
        if let lex_4_25::Token::OpenBrace = lex_4_25::next_token(chars, cur) {
            result.children.push(Box::new(is_node));
        } else {
            panic!("Invalid contract definition")
        }
    }
    result.children.push(Box::new(parse_contract_part(chars, cur)));
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::CloseBrace => (),
        _ => panic!("Invalid contract definition")
    }
    result
}

fn parse_contract_part(chars: &Vec<char>, cur: &mut usize) -> ParseNode {
    let mut result = lex_4_25::Token::OpenBrace.to_leaf(); 
    match lex_4_25::peek_token(chars, cur) {
        lex_4_25::Token::Enum => result.children.push(Box::new(parse_enum_definition(chars, cur))),
        _ => () 
    }
    result
}

fn parse_enum_definition(chars: &Vec<char>, cur: &mut usize) -> ParseNode {
    let mut result = lex_4_25::Token::Enum.to_leaf();
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::Enum => (),
        _ => panic!("Invalid enum definition")
    }
    match lex_4_25::next_token(chars, cur) {
        id @ lex_4_25::Token::Identifier(..) => result.add_child(id),
        _ => panic!("Invalid enum definition")
    }
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::OpenBrace => result.add_child(lex_4_25::Token::OpenBrace),
        _ => panic!("Invalid enum definition")
    }
    let mut stop = false;
    while !stop {
        match lex_4_25::peek_token(chars, cur) {
            lex_4_25::Token::Identifier(..) => result.children[1].add_child(lex_4_25::next_token(chars, cur)),
            _ => stop = true
        }
        if !stop {
            if let lex_4_25::Token::Comma = lex_4_25::peek_token(chars, cur) {
                lex_4_25::next_token(chars, cur);
            } else {
                stop = true;
            }
        }
    }
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::CloseBrace => (),
        _ => panic!("Invalid enum definition")
    }
    result
}

fn parse_inheritance_specifier(chars: &Vec<char>, cur: &mut usize) -> ParseNode { 
    let mut result = lex_4_25::Token::OpenParenthesis.to_leaf();
    result.children.push(Box::new(parse_user_defined_type_name(chars, cur)));
    if let lex_4_25::Token::OpenParenthesis = lex_4_25::peek_token(chars, cur) {
        lex_4_25::next_token(chars, cur);
        result.children.push(Box::new(parse_expression_list(chars, cur)));
        match lex_4_25::next_token(chars, cur) {
            lex_4_25::Token::CloseParenthesis => (),
            _ => panic!("Invalid inheritance specifier")
        }
    }
    result
}

fn parse_function_definition(chars: &Vec<char>, cur: &mut usize) -> ParseNode {
    let mut result = lex_4_25::Token::Function.to_leaf();
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::Function => (),
        _ => panic!("Invalid function definition")
    }
    result
}

/*** Expression ***/

fn parse_operation(chars: &Vec<char>, cur: &mut usize, left: ParseNode) -> ParseNode {
    let mut result = ParseNode{ node: lex_4_25::Token::NoMatch, children: vec![] };
    match lex_4_25::peek_token(&chars, cur) {
        lex_4_25::Token::Decrement | lex_4_25::Token::Increment => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            return parse_operation(&chars, cur, result);
        }
        lex_4_25::Token::OpenBracket => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::OpenBracket         |
                lex_4_25::Token::Dot                 |
                lex_4_25::Token::Power               |
                lex_4_25::Token::Divide              |
                lex_4_25::Token::Minus               |
                lex_4_25::Token::Modulus             |
                lex_4_25::Token::Multiply            |
                lex_4_25::Token::Plus                |
                lex_4_25::Token::ShiftLeft           |
                lex_4_25::Token::ShiftRight          |
                lex_4_25::Token::BitwiseAnd          |
                lex_4_25::Token::BitwiseXor          |
                lex_4_25::Token::BitwiseOr           |
                lex_4_25::Token::GreaterThan         |
                lex_4_25::Token::GreaterThanOrEquals |
                lex_4_25::Token::LessThan            |
                lex_4_25::Token::LessThanOrEquals    |
                lex_4_25::Token::Equals              |
                lex_4_25::Token::NotEquals           |
                lex_4_25::Token::LogicalAnd          |
                lex_4_25::Token::LogicalOr           |
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                _ => result.children.push(Box::new(right))
            }
            match lex_4_25::next_token(&chars, cur) {
                lex_4_25::Token::CloseBracket => return parse_operation(&chars, cur, result),
                _ => panic!("Invalid array access")
            }
        }
        lex_4_25::Token::Dot => {
            result.node = lex_4_25::next_token(&chars, cur); 
            result.children.push(Box::new(left));
            let right = lex_4_25::next_token(&chars, cur);
            match right {
                id @ lex_4_25::Token::Identifier(..) => result.add_child(id),
                _ => panic!("Invalid member access")
            }
            return parse_operation(&chars, cur, result);
        }
        lex_4_25::Token::OpenParenthesis => {
            result.node = lex_4_25::Token::Function;
            result.children.push(Box::new(left));
            result.children.push(Box::new(parse_function_call_arguments(&chars, cur)));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::Power               |
                lex_4_25::Token::Divide              |
                lex_4_25::Token::Minus               |
                lex_4_25::Token::Modulus             |
                lex_4_25::Token::Multiply            |
                lex_4_25::Token::Plus                |
                lex_4_25::Token::ShiftLeft           |
                lex_4_25::Token::ShiftRight          |
                lex_4_25::Token::BitwiseAnd          |
                lex_4_25::Token::BitwiseXor          |
                lex_4_25::Token::BitwiseOr           |
                lex_4_25::Token::GreaterThan         |
                lex_4_25::Token::GreaterThanOrEquals |
                lex_4_25::Token::LessThan            |
                lex_4_25::Token::LessThanOrEquals    |
                lex_4_25::Token::Equals              |
                lex_4_25::Token::NotEquals           |
                lex_4_25::Token::LogicalAnd          |
                lex_4_25::Token::LogicalOr           |
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                lex_4_25::Token::NoMatch => (),
                _ => result.children.push(Box::new(right))
            }
        }
        lex_4_25::Token::Power => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::Power               |
                lex_4_25::Token::Divide              |
                lex_4_25::Token::Minus               |
                lex_4_25::Token::Modulus             |
                lex_4_25::Token::Multiply            |
                lex_4_25::Token::Plus                |
                lex_4_25::Token::ShiftLeft           |
                lex_4_25::Token::ShiftRight          |
                lex_4_25::Token::BitwiseAnd          |
                lex_4_25::Token::BitwiseXor          |
                lex_4_25::Token::BitwiseOr           |
                lex_4_25::Token::GreaterThan         |
                lex_4_25::Token::GreaterThanOrEquals |
                lex_4_25::Token::LessThan            |
                lex_4_25::Token::LessThanOrEquals    |
                lex_4_25::Token::Equals              |
                lex_4_25::Token::NotEquals           |
                lex_4_25::Token::LogicalAnd          |
                lex_4_25::Token::LogicalOr           |
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                _ => result.children.push(Box::new(right))
            }
        }
        lex_4_25::Token::Divide   |
        lex_4_25::Token::Multiply | 
        lex_4_25::Token::Modulus => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::Divide              |
                lex_4_25::Token::Minus               |
                lex_4_25::Token::Modulus             |
                lex_4_25::Token::Multiply            |
                lex_4_25::Token::Plus                |
                lex_4_25::Token::ShiftLeft           |
                lex_4_25::Token::ShiftRight          |
                lex_4_25::Token::BitwiseAnd          |
                lex_4_25::Token::BitwiseXor          |
                lex_4_25::Token::BitwiseOr           |
                lex_4_25::Token::GreaterThan         |
                lex_4_25::Token::GreaterThanOrEquals |
                lex_4_25::Token::LessThan            |
                lex_4_25::Token::LessThanOrEquals    |
                lex_4_25::Token::Equals              |
                lex_4_25::Token::NotEquals           |
                lex_4_25::Token::LogicalAnd          |
                lex_4_25::Token::LogicalOr           |
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                _ => result.children.push(Box::new(right))
            }
        }
        lex_4_25::Token::Plus | lex_4_25::Token::Minus => {
            result.node = lex_4_25::next_token(&chars, cur); 
            result.children.push(Box::new(left));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::Minus               |
                lex_4_25::Token::Plus                |
                lex_4_25::Token::ShiftLeft           |
                lex_4_25::Token::ShiftRight          |
                lex_4_25::Token::BitwiseAnd          |
                lex_4_25::Token::BitwiseXor          |
                lex_4_25::Token::BitwiseOr           |
                lex_4_25::Token::GreaterThan         |
                lex_4_25::Token::GreaterThanOrEquals |
                lex_4_25::Token::LessThan            |
                lex_4_25::Token::LessThanOrEquals    |
                lex_4_25::Token::Equals              |
                lex_4_25::Token::NotEquals           |
                lex_4_25::Token::LogicalAnd          |
                lex_4_25::Token::LogicalOr           |
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                _ => result.children.push(Box::new(right))
            }
        }
        lex_4_25::Token::ShiftLeft | lex_4_25::Token::ShiftRight => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::ShiftLeft           |
                lex_4_25::Token::ShiftRight          |
                lex_4_25::Token::BitwiseAnd          |
                lex_4_25::Token::BitwiseXor          |
                lex_4_25::Token::BitwiseOr           |
                lex_4_25::Token::GreaterThan         |
                lex_4_25::Token::GreaterThanOrEquals |
                lex_4_25::Token::LessThan            |
                lex_4_25::Token::LessThanOrEquals    |
                lex_4_25::Token::Equals              |
                lex_4_25::Token::NotEquals           |
                lex_4_25::Token::LogicalAnd          |
                lex_4_25::Token::LogicalOr           |
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                _ => result.children.push(Box::new(right))
            }
        }
        lex_4_25::Token::BitwiseAnd => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::BitwiseAnd          |
                lex_4_25::Token::BitwiseXor          |
                lex_4_25::Token::BitwiseOr           |
                lex_4_25::Token::GreaterThan         |
                lex_4_25::Token::GreaterThanOrEquals |
                lex_4_25::Token::LessThan            |
                lex_4_25::Token::LessThanOrEquals    |
                lex_4_25::Token::Equals              |
                lex_4_25::Token::NotEquals           |
                lex_4_25::Token::LogicalAnd          |
                lex_4_25::Token::LogicalOr           |
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                _ => result.children.push(Box::new(right))
            }
        }
        lex_4_25::Token::BitwiseXor => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::BitwiseXor          |
                lex_4_25::Token::BitwiseOr           |
                lex_4_25::Token::GreaterThan         |
                lex_4_25::Token::GreaterThanOrEquals |
                lex_4_25::Token::LessThan            |
                lex_4_25::Token::LessThanOrEquals    |
                lex_4_25::Token::Equals              |
                lex_4_25::Token::NotEquals           |
                lex_4_25::Token::LogicalAnd          |
                lex_4_25::Token::LogicalOr           |
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                _ => result.children.push(Box::new(right))
            }
        }
        lex_4_25::Token::BitwiseOr => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::BitwiseXor          |
                lex_4_25::Token::BitwiseOr           |
                lex_4_25::Token::GreaterThan         |
                lex_4_25::Token::GreaterThanOrEquals |
                lex_4_25::Token::LessThan            |
                lex_4_25::Token::LessThanOrEquals    |
                lex_4_25::Token::Equals              |
                lex_4_25::Token::NotEquals           |
                lex_4_25::Token::LogicalAnd          |
                lex_4_25::Token::LogicalOr           |
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                _ => result.children.push(Box::new(right))
            }
        }
        lex_4_25::Token::GreaterThan         |
        lex_4_25::Token::LessThan            |
        lex_4_25::Token::GreaterThanOrEquals |
        lex_4_25::Token::LessThanOrEquals => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::GreaterThan         |
                lex_4_25::Token::GreaterThanOrEquals |
                lex_4_25::Token::LessThan            |
                lex_4_25::Token::LessThanOrEquals    |
                lex_4_25::Token::Equals              |
                lex_4_25::Token::NotEquals           |
                lex_4_25::Token::LogicalAnd          |
                lex_4_25::Token::LogicalOr           |
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                _ => result.children.push(Box::new(right))
            }
        }
        lex_4_25::Token::Equals | lex_4_25::Token::NotEquals => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::Equals              |
                lex_4_25::Token::NotEquals           |
                lex_4_25::Token::LogicalAnd          |
                lex_4_25::Token::LogicalOr           |
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                _ => result.children.push(Box::new(right))
            }
        }
        lex_4_25::Token::LogicalAnd | lex_4_25::Token::LogicalOr => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::LogicalOr           | 
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                _ => result.children.push(Box::new(right))
            }
        }
        lex_4_25::Token::Question | lex_4_25::Token::Colon => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::Question            |
                lex_4_25::Token::Colon               |
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                _ => result.children.push(Box::new(right))
            }
        }
        lex_4_25::Token::Set              |
        lex_4_25::Token::OrEquals         |
        lex_4_25::Token::XorEquals        |
        lex_4_25::Token::AndEquals        |
        lex_4_25::Token::ShiftLeftEquals  |
        lex_4_25::Token::ShiftRightEquals |
        lex_4_25::Token::PlusEquals       |
        lex_4_25::Token::MinusEquals      |
        lex_4_25::Token::ModEquals        |
        lex_4_25::Token::MultiplyEquals   |
        lex_4_25::Token::DivideEquals => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            let right = parse_expression(&chars, cur);
            match right.node {
                lex_4_25::Token::Set                 |
                lex_4_25::Token::OrEquals            |
                lex_4_25::Token::XorEquals           |
                lex_4_25::Token::AndEquals           |
                lex_4_25::Token::ShiftLeftEquals     |
                lex_4_25::Token::ShiftRightEquals    |
                lex_4_25::Token::PlusEquals          |
                lex_4_25::Token::MinusEquals         |
                lex_4_25::Token::MultiplyEquals      |
                lex_4_25::Token::DivideEquals        |
                lex_4_25::Token::ModEquals => result = result.merge_expressions(right), 
                _ => result.children.push(Box::new(right))
            }
        }
        _ => {
            result = left;
        }
    }
    result
}

fn parse_function_call_arguments(chars: &Vec<char>, cur: &mut usize) -> ParseNode {
    let result;
    let mut next = lex_4_25::next_token(&chars, cur);
    match next {
        lex_4_25::Token::OpenParenthesis => (),
        _ => panic!("Invalid function call")
    }
    next = lex_4_25::peek_token(&chars, cur);
    match next {
        lex_4_25::Token::OpenBrace => {
            lex_4_25::next_token(&chars, cur);
            result = parse_name_value_list(&chars, cur);
            match lex_4_25::next_token(&chars, cur) {
                lex_4_25::Token::CloseBrace => (),
                _ => panic!("Invalid function call")
            }
        }
        _ => result = parse_expression_list(&chars, cur)
    }
    match lex_4_25::next_token(&chars, cur) {
        lex_4_25::Token::CloseParenthesis => return result,
        _ => panic!("Invalid function call")
    }
}

fn parse_name_value_list(chars: &Vec<char>, cur: &mut usize) -> ParseNode {
    let mut result = ParseNode{ node: lex_4_25::Token::OpenBrace, children: vec![] };
    let mut stop = false;
    while !stop {
        let mut child = ParseNode { node: lex_4_25::Token::Colon, children: vec![] };
        match lex_4_25::peek_token(&chars, cur) {
            lex_4_25::Token::Identifier(..) => {
                child.add_child(lex_4_25::next_token(&chars, cur));
            }
            _ => stop = true
        }
        if !stop {
            match lex_4_25::next_token(&chars, cur) {
                lex_4_25::Token::Colon => (),
                _ => panic!("Invalid name value list")
            }
            child.children.push(Box::new(parse_expression(&chars, cur)));
            match lex_4_25::peek_token(&chars, cur) {
                lex_4_25::Token::Comma => (),
                _ => stop = true
            }
            result.children.push(Box::new(child));
        }
    }
    result
}

pub fn parse_expression(chars: &Vec<char>, cur: &mut usize) -> ParseNode {
    let mut result = ParseNode{ node: lex_4_25::Token::NoMatch, children: vec![] };
    let mut peek = lex_4_25::peek_token(&chars, cur);
    match peek {
        lex_4_25::Token::New => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(parse_type_name(&chars, cur)));
        }
        lex_4_25::Token::DecimalNumber(..) | lex_4_25::Token::HexNumber(..) => {
            let mut left = lex_4_25::next_token(&chars, cur).to_leaf();
            peek = lex_4_25::peek_token(&chars, cur);
            if peek.is_number_unit() {
                 left.add_child(lex_4_25::next_token(&chars, cur));  
            }
            result = parse_operation(&chars, cur, left);
        }
        lex_4_25::Token::Identifier(..)    |
        lex_4_25::Token::HexLiteral(..)    |
        lex_4_25::Token::StringLiteral(..) |
        lex_4_25::Token::True              |
        lex_4_25::Token::False => {
            let left = lex_4_25::next_token(&chars, cur);
            result = parse_operation(&chars, cur, left.to_leaf());
        }
        lex_4_25::Token::OpenParenthesis => {
            lex_4_25::next_token(&chars, cur);
            result = parse_expression(&chars, cur);
            let mut stop = false;
            while !stop {
                match lex_4_25::next_token(&chars, cur) {
                    lex_4_25::Token::Comma => result.children.push(Box::new(parse_expression(&chars, cur))),
                    lex_4_25::Token::CloseParenthesis => stop = true,
                    _ => panic!("Invalid tuple expression")
                }
                result = parse_operation(&chars, cur, result);
            }
        }
        lex_4_25::Token::Exclamation | 
        lex_4_25::Token::Tilda       | 
        lex_4_25::Token::Delete      | 
        lex_4_25::Token::Increment   | 
        lex_4_25::Token::Decrement   |
        lex_4_25::Token::Plus        |
        lex_4_25::Token::Minus => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(parse_expression(&chars, cur)));
        }
        elementary => {
            if elementary.is_elementary_type() {
                let left = lex_4_25::next_token(&chars, cur);
                result = parse_operation(&chars, cur, left.to_leaf());
            }
        }
    }
    result
}

fn parse_expression_list(chars: &Vec<char>, cur: &mut usize) -> ParseNode {
    let mut result = lex_4_25::Token::OpenParenthesis.to_leaf();
    let mut stop = false;
    while !stop {
        let returned = parse_expression(&chars, cur);
        match returned.node {
            lex_4_25::Token::NoMatch => stop = true,
            _ => result.children.push(Box::new(returned))
        }
        if !stop {
            if let lex_4_25::Token::Comma = lex_4_25::peek_token(&chars, cur) {
                lex_4_25::next_token(&chars, cur);
            } else {
                stop = true;
            }
        }
    }
    result
}

/*** Types ***/

pub fn parse_type_name(chars: &Vec<char>, cur: &mut usize) -> ParseNode {
    return match lex_4_25::peek_token(chars, cur) {
        lex_4_25::Token::Identifier(..) => parse_user_defined_type_name(chars, cur),
        lex_4_25::Token::Mapping => parse_mapping(chars, cur),
        lex_4_25::Token::Function => parse_function_type_name(chars, cur),
        elementary => {
            if elementary.is_elementary_type() {
                lex_4_25::next_token(chars, cur);
                // Try to parse an array type and return elementary if the type isn't followed by
                // array brackets.
                parse_array_type_name(chars, cur, elementary.to_leaf())
            } else {
                panic!("Invalid type name")
            }
        }
    }
}

fn parse_user_defined_type_name(chars: &Vec<char>, cur: &mut usize) -> ParseNode {
    let mut result = lex_4_25::Token::Dot.to_leaf();
    let mut stop = false;
    while !stop {
        match lex_4_25::next_token(chars, cur) {
            id @ lex_4_25::Token::Identifier(..) => result.add_child(id),
            _ => panic!("Invalid user defined type name")
        }
        if !stop {
            if let lex_4_25::Token::Dot = lex_4_25::peek_token(&chars, cur) {
                lex_4_25::next_token(&chars, cur);
            } else {
                stop = true;
            }
        }
    }
    result
}

fn parse_mapping(chars: &Vec<char>, cur: &mut usize) -> ParseNode { 
    let mut result = lex_4_25::Token::Mapping.to_leaf();
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::Mapping => (),
        _ => panic!("Invalid mapping")
    }
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::OpenParenthesis => (),
        _ => panic!("Invalid mapping")
    }
    let elementary = lex_4_25::next_token(chars, cur);
    if elementary.is_elementary_type() {
        result.add_child(elementary);
    } else {
        panic!("Invalid mapping");
    }
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::Arrow => (),
        _ => panic!("Invalid mapping")
    }
    result.children.push(Box::new(parse_type_name(chars, cur)));
    println!("{:?}", result);
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::CloseParenthesis => (),
        _ => panic!("Invalid mapping")
    }
    result
}

fn parse_function_type_name(chars: &Vec<char>, cur: &mut usize) -> ParseNode { 
    let mut result = lex_4_25::Token::Function.to_leaf();
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::Function => (),
        _ => panic!("Invalid function type")
    }
    result.children.push(Box::new(parse_function_type_parameter_list(chars, cur)));
    let mut start = false;
    while !start {
        match lex_4_25::peek_token(chars, cur) {
            lex_4_25::Token::Internal |
            lex_4_25::Token::External | 
            lex_4_25::Token::Pure     |
            lex_4_25::Token::Constant |
            lex_4_25::Token::View     |
            lex_4_25::Token::Payable => result.add_child(lex_4_25::next_token(chars, cur)),
            _ => start = true
        }
    }
    match lex_4_25::peek_token(chars, cur) {
        lex_4_25::Token::Returns => {
            result.add_child(lex_4_25::next_token(chars, cur));
            let last = result.children.len() - 1;
            result.children[last].children.push(Box::new(parse_function_type_parameter_list(chars, cur)));
        }
        _ => () 
    }
    result
}

fn parse_function_type_parameter_list(chars: &Vec<char>, cur: &mut usize) -> ParseNode {
    let mut result = lex_4_25::Token::OpenParenthesis.to_leaf();
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::OpenParenthesis => (),
        _ => panic!("Invalid function type parameter list")
    }
    let mut stop = false;
    while !stop {
        match lex_4_25::peek_token(chars, cur) {
            lex_4_25::Token::CloseParenthesis => stop = true,
            _ => result.children.push(Box::new(parse_type_name(chars, cur)))
        }
        match lex_4_25::peek_token(chars, cur) {
            lex_4_25::Token::Comma => {
                lex_4_25::next_token(chars, cur);
            }
            _ => stop = true,
        }
    }
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::CloseParenthesis => (),
        _ => panic!("Invalid function type parameter list")
    }
    result
}

fn parse_array_type_name(chars: &Vec<char>, cur: &mut usize, left: ParseNode) -> ParseNode { 
    let mut result = lex_4_25::Token::OpenBracket.to_leaf();
    match lex_4_25::peek_token(chars, cur) {
        lex_4_25::Token::OpenBracket => {
            lex_4_25::next_token(chars, cur);
        }
        _ => return left 
    }
    result.children.push(Box::new(left));
    let right = parse_expression(chars, cur);
    match right.node {
        lex_4_25::Token::NoMatch => (),
        _ => result.children.push(Box::new(right))
    }
    match lex_4_25::next_token(chars, cur) {
        lex_4_25::Token::CloseBracket => (),
        _ => panic!("Invalid array type name")
    }
    result
}
