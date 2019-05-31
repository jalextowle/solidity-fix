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

/* Methods */
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
}

impl lex_4_25::Token {
    fn to_leaf(mut self) -> ParseNode {
        ParseNode { node: self, children: vec![] }
    }
}

pub fn parse(input: String) -> ParseTree {
    let current_node = &mut ParseNode{ node: lex_4_25::Token::NoMatch, children: vec![] }; 
    let mut tree = ParseTree{ children: vec![] };
    let mut cur = 0;
    let input_chars = input.chars().collect::<Vec<char>>(); 
    while cur < input_chars.len() {
        let next = lex_4_25::next_token(&input_chars, &mut cur);
        match next {
            lex_4_25::Token::Pragma => {
                current_node.node = lex_4_25::Token::Pragma;
                parse_pragma(&input_chars, &mut cur, current_node, &mut tree);
            }
            lex_4_25::Token::Import => {
                // TODO
            }
            lex_4_25::Token::Contract => {
                current_node.node = lex_4_25::Token::Contract;
                parse_contract(&input_chars, &mut cur, current_node, &mut tree);
            }
            lex_4_25::Token::Library => {
                current_node.node = lex_4_25::Token::Library;
                parse_contract(&input_chars, &mut cur, current_node, &mut tree);
            }
            lex_4_25::Token::Interface => {
                current_node.node = lex_4_25::Token::Interface;
                parse_contract(&input_chars, &mut cur, current_node, &mut tree);
            }
            _ => {
                // TODO: Add the below when everything is implemented
                /* panic!("Invalid top level expression: {:?}", none) */
            }
        }
    }
    tree
}

fn parse_pragma(chars: &Vec<char>, cur: &mut usize, node: &mut ParseNode, tree: &mut ParseTree) { 
    let mut next = lex_4_25::next_token(chars, cur);
    match next {
        lex_4_25::Token::Identifier(name) => {
            if name != "solidity" {
                panic!("Invalid source file: Not a solidity file")
            }
            node.add_child(lex_4_25::Token::Identifier(name));
        }
        _ => panic!("Invalid pragma declaration")
    }
    next = lex_4_25::next_token(&chars, cur);
    match next {
        lex_4_25::Token::Version(version) => {
            if version != "^0.4.25" && version != "0.4.25" {
                panic!("Invalid source file: version other than 0.4.25 specfied")
            }
            node.add_child(lex_4_25::Token::Version(version));
        }
        _ => panic!("Invalid pragma declaration")
    }
    next = lex_4_25::next_token(&chars, cur);
    match next {
        lex_4_25::Token::Semicolon => tree.children.push((*node).clone()), 
        _ => panic!("Invalid pragma declaration")
    }
} 

fn parse_import(chars: &Vec<char>, cur: &mut usize, node: &mut ParseNode, tree: &mut ParseTree) { }

fn parse_contract(chars: &Vec<char>, cur: &mut usize, node: &mut ParseNode, tree: &mut ParseTree) { 
    let mut next = lex_4_25::next_token(&chars, cur);
    match next {
        id@lex_4_25::Token::Identifier(..) => node.add_child(id),
        _ => panic!("Invalid contract definition")
    }
    next = lex_4_25::next_token(&chars, cur);
    match next {
        lex_4_25::Token::Is => { 
            node.add_child(lex_4_25::Token::Is);
            // TODO: parse_inheritance_specifier(chars, cur, node, tree);
        }
        lex_4_25::Token::OpenBrace => (),
        _ => panic!("Invalid contract definition")
    }
    parse_contract_part(chars, cur, node, tree);
    tree.children.push((*node).clone());
}

/*** Contract Part ***/

fn parse_contract_part(chars: &Vec<char>, cur: &mut usize, node: &mut ParseNode, tree: &mut ParseTree) {
    let mut next = lex_4_25::next_token(&chars, cur);
    match next {
        lex_4_25::Token::Using => {
            node.add_child(lex_4_25::Token::Using);
            // TODO: parse_using_for_declaration(chars, cur, node, tree);
        }
        lex_4_25::Token::Struct => {
            node.add_child(lex_4_25::Token::Struct);
            // TODO: parse_struct_definition(chars, cur, node, tree);
        }
        lex_4_25::Token::Modifier => {
            node.add_child(lex_4_25::Token::Modifier);
            // TODO: parse_modifier_definition(chars, cur, node, tree);
        }
        lex_4_25::Token::Function => {
            node.add_child(lex_4_25::Token::Function);
            // TODO: parse_function_definition(chars, cur, node, tree);
        }
        lex_4_25::Token::Event => {
            node.add_child(lex_4_25::Token::Event);
            // TODO: parse_event_definition(chars, cur, node, tree);
        }
        lex_4_25::Token::Enum => {
            node.add_child(lex_4_25::Token::Enum);
            // TODO: parse_enum_definition(chars, cur, node, tree);
        }
        matched => ()// TODO: parse_state_variable_declaration(chars, cur, node, tree, matched)
    }
}

/*** Expression ***/

fn parse_operation(chars: &Vec<char>, cur: &mut usize, left: ParseNode) -> ParseNode {
    let mut result = ParseNode{ node: lex_4_25::Token::NoMatch, children: vec![] };
    let mut peek = lex_4_25::peek_token(&chars, cur);
    match peek {
        lex_4_25::Token::Decrement | lex_4_25::Token::Increment => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            return parse_operation(&chars, cur, result);
        }
        lex_4_25::Token::OpenBracket => {
            result.node = lex_4_25::next_token(&chars, cur);
            result.children.push(Box::new(left));
            let mut right = parse_expression(&chars, cur);
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
            let mut right = lex_4_25::next_token(&chars, cur);
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
            let mut right = parse_expression(&chars, cur);
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
            let mut right = parse_expression(&chars, cur);
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
            let mut right = parse_expression(&chars, cur);
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
            let mut left = ParseNode { node: lex_4_25::next_token(&chars, cur), children: vec![] };
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
    let mut result = ParseNode{ node: lex_4_25::Token::OpenParenthesis, children: vec![] };
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

fn parse_type_name(chars: &Vec<char>, cur: &mut usize) -> ParseNode {
    ParseNode{ node: lex_4_25::Token::NoMatch, children: vec![] }
}
