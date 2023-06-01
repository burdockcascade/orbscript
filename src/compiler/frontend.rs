use peg::parser;

use crate::compiler::token::Token;

parser!(pub grammar parser() for str {

    //==============================================================================================
    // TOP LEVEL STATEMENTS

    // top level rule
    pub rule script() -> Vec<Token>
        = WHITESPACE() f:(constant() / comment() / function())* WHITESPACE() { f }

    rule statement() -> Token
        = WHITESPACE() s:(
            print() /
            comment() /
            var() /
            assert() /
            call() /
            rtn() /
            loop_while() /
            loop_for() /
            loop_for_each() /
            if_else() /
            assignment()
        ) WHITESPACE() { s }

    rule print() -> Token
        = "print" _ e:expression() { Token::Print(Box::new(e)) }

    rule assert() -> Token
        = "assert" _ e:expression() { Token::Assert(Box::new(e)) }

    rule comment() -> Token
        = "--" s:$([' ' | ',' |'a'..='z' | 'A'..='Z' | '0'..='9']*) NEWLINES() { Token::Comment(s.to_owned()) }

    //==============================================================================================
    // VARIABLES

    // variable declaration either with a value or default to null
    rule var() -> Token
        = "var" _ i:identifier() WHITESPACE() "=" WHITESPACE() e:expression() {  Token::Variable(Box::new(i), Box::new(e)) } /
          "var" _ i:identifier() { Token::Variable(Box::new(i), Box::new(Token::Null)) }

    // existing variable assignment
    rule assignment() -> Token
        = left:(array_index() / identifier()) WHITESPACE() "=" WHITESPACE() r:expression() {  Token::Assign(Box::new(left), Box::new(r)) }

    rule constant() -> Token
        = "const" _ i:identifier() WHITESPACE() "=" WHITESPACE() e:expression() NEWLINES() {  Token::Constant(Box::new(i), Box::new(e)) }

    //==============================================================================================
    // FUNCTIONS

    // function definition with parameters
    rule function() -> Token
        = "function" _ name:identifier() _ "(" params:param_list() ")" stmts:statement()* WHITESPACE() "end" WHITESPACE() { Token::Function(name.to_string(), params, stmts) }

    rule lambda() -> Token
        = "function(" params:param_list() ")" stmts:statement()* WHITESPACE() "end" WHITESPACE() { Token::AnonFunction(params, stmts) }

    // function call with arguments
    rule call() -> Token
        = i:identifier() "(" args:arg_list() ")" { Token::Call(Box::new(i), args) }

    // argument list
    rule arg_list() -> Vec<Token>
        = quiet!{args:((_ e:expression() _ {e}) ** ",") { args } }

    // parameter list
    rule param_list() -> Vec<Token>
        = quiet!{args:((_ e:identifier() _ {e}) ** ",") { args } }

    rule rtn() -> Token
        = "return" _ e:expression() { Token::Return(Box::new(e)) }

    //==============================================================================================
    // LOOPS

    rule loop_while() -> Token
        = "while" _ e:expression() _ "do" _ stmts:statement()* _ "end" { Token::WhileLoop(Box::new(e), stmts) }

    rule loop_for() -> Token
        = "for" _ i:identifier() _ "=" _ start:expression() _ "to" _ end:expression() _ "do" _ stmts:statement()* _ "end" { Token::ForI(Box::new(i), Box::new(start), Box::new(Token::Integer(1)), Box::new(end), stmts) }
        / "for" _ i:identifier() _ "=" _ start:expression() _ "to" _ end:expression() _ "step" _ step:expression() _ "do" _ stmts:statement()* _ "end" { Token::ForI(Box::new(i), Box::new(start), Box::new(step), Box::new(end), stmts) }

    rule loop_for_each() -> Token
        = "for" _ i:identifier() _ "in" _ e:expression() _ "do" _ stmts:statement()* _ "end" { Token::ForEach(Box::new(i), Box::new(e), stmts) }

    //==============================================================================================
    // IF

    rule if_else() -> Token
        = "if" _ e:expression() WHITESPACE() "then" WHITESPACE() then_body:statement()* WHITESPACE() WHITESPACE()
            else_body:("else" WHITESPACE() s:statement()* WHITESPACE()  { s })? WHITESPACE() "end"
        { Token::IfElse(Box::new(e), then_body, else_body) }

    //==============================================================================================
    // EXPRESSIONS

    rule expression() -> Token = precedence!{
        a:@ _ "==" _ b:(@) { Token::Eq(Box::new(a), Box::new(b)) }
        a:@ _ "!=" _ b:(@) { Token::Ne(Box::new(a), Box::new(b)) }
        a:@ _ "<"  _ b:(@) { Token::Lt(Box::new(a), Box::new(b)) }
        a:@ _ "<=" _ b:(@) { Token::Le(Box::new(a), Box::new(b)) }
        a:@ _ ">"  _ b:(@) { Token::Gt(Box::new(a), Box::new(b)) }
        a:@ _ ">=" _ b:(@) { Token::Ge(Box::new(a), Box::new(b)) }
        --
        a:@ _ "+" _ b:(@) { Token::Add(Box::new(a), Box::new(b)) }
        a:@ _ "-" _ b:(@) { Token::Sub(Box::new(a), Box::new(b)) }
        --
        a:@ _ "*" _ b:(@) { Token::Mul(Box::new(a), Box::new(b)) }
        a:@ _ "/" _ b:(@) { Token::Div(Box::new(a), Box::new(b)) }
        a:@ _ "^" _ b:(@) { Token::Pow(Box::new(a), Box::new(b)) }
        --
        l:literal() { l }
    }

    rule literal() -> Token
        = f:float() { Token::Float(f) }
        / i:integer() { Token::Integer(i) }
        / list()
        / dictionary()
        / array_index()
        / l:lambda() { l } // this needs to come before call
        / c:call() { c }
        / n:null() { n }
        / b:boolean() { b }
        / s:string() { s }
        / i:identifier() { i } // this is greedy and must always come last


    rule identifier() -> Token
        = n:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) { Token::Identifier(n.to_owned()) }

    rule string() -> Token
        = "\""  n:$([^'"']*) "\""  { Token::String(n.to_owned()) }

    rule integer() -> i32
        = n:$("-"? ['0'..='9']+) { n.parse().unwrap() }

    rule float() -> f32
        = n:$("-"? ['0'..='9']+ "." ['0'..='9']+) { n.parse().unwrap() }

    rule boolean() -> Token
        = "true" { Token::Bool(true) }
        / "false" { Token::Bool(false) }

    rule list() -> Token
        = quiet!{ "[" WHITESPACE() elements:(( WHITESPACE() e:expression() WHITESPACE() {e}) ** ",") WHITESPACE() "]" { Token::Array(elements) } }

    rule dictionary() -> Token
        = "{" WHITESPACE() kv:(( WHITESPACE() k:string() WHITESPACE() ":" WHITESPACE() e:expression() WHITESPACE() {  Token::KeyValuePair(k.to_string(), Box::new(e)) } ) ** ",") WHITESPACE() "}" { Token::Dictionary(kv) }

    rule array_index() -> Token
        =  i:identifier() "[" WHITESPACE() e:expression() WHITESPACE() "]" { Token::CollectionIndex(Box::new(i), Box::new(e)) }

    rule null() -> Token
        = "null" { Token::Null }

    //==============================================================================================
    // WHITESPACE

    rule _() =  quiet!{[' ' | '\t']*}
    rule NEWLINE() = quiet!{ ['\n'|'\r'] }
    rule NEWLINES() = quiet!{ ['\n'|'\r']* }
    rule WHITESPACE() = quiet!{ [' '|'\t'|'\n'|'\r']* }
    rule UTF8CHAR() -> char = quiet!{ c:([^ '\x00'..='\x1F' | '\t' | '\n'|'\r']) { c } }

});