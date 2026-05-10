// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Type expression Pratt parser.
//!
//! Mirrors `crates/ox_jsdoc/src/parser/type_parse.rs` but builds
//! [`TypeNodeData`] (heap `Box`) instead of arena-allocated `TypeNode`.
//! The Pratt parser logic itself is identical.

use oxc_span::Span;

use super::context::ParserContext;
use super::diagnostics::TypeDiagnosticKind;
use super::lexer::Lexer;
use super::precedence::Precedence;
use super::token::TokenKind;
use super::type_data::*;

impl<'arena, 'a> ParserContext<'arena, 'a> {
    /// Parse `{type}` text into a [`TypeNodeData`] expression.
    pub(crate) fn parse_type_expression(
        &mut self,
        type_text: &'a str,
        type_base_offset: u32,
        mode: ParseMode,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let mut lexer = Lexer::new(type_text, type_base_offset, mode.is_loose());
        let mut disallow_conditional = false;
        let result =
            self.parse_type_pratt(&mut lexer, mode, &mut disallow_conditional, Precedence::All)?;
        if lexer.current.kind != TokenKind::EOF {
            self.type_diag(TypeDiagnosticKind::EarlyEndOfParse);
            return None;
        }
        Some(result)
    }

    fn parse_type_pratt(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        min_precedence: Precedence,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let mut left = self.parse_prefix_type(lexer, mode, disallow_conditional)?;
        loop {
            if *disallow_conditional && lexer.current.kind == TokenKind::Question {
                break;
            }
            let infix_prec = self.cur_infix_precedence(lexer, mode);
            if infix_prec <= min_precedence {
                break;
            }
            left = self.parse_infix_type(lexer, mode, disallow_conditional, left)?;
        }
        Some(left)
    }

    fn parse_prefix_type(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        match lexer.current.kind {
            TokenKind::Identifier | TokenKind::This => self.parse_name(lexer, mode),
            TokenKind::New if mode.is_typescript() && lexer.next.kind == TokenKind::LParen => {
                self.parse_new_function(lexer, mode, disallow_conditional)
            }
            TokenKind::New => self.parse_name(lexer, mode),
            TokenKind::Keyof if !mode.is_typescript() => self.parse_name(lexer, mode),
            TokenKind::Event | TokenKind::External | TokenKind::In
                if mode.is_typescript() || mode.is_closure() =>
            {
                self.parse_name(lexer, mode)
            }
            TokenKind::Null => {
                let token = lexer.current;
                lexer.bump();
                Some(Box::new(TypeNodeData::Null(TypeNull {
                    span: Span::new(token.start, token.end),
                })))
            }
            TokenKind::Undefined => {
                let token = lexer.current;
                lexer.bump();
                Some(Box::new(TypeNodeData::Undefined(TypeUndefined {
                    span: Span::new(token.start, token.end),
                })))
            }
            TokenKind::Star => {
                let token = lexer.current;
                lexer.bump();
                Some(Box::new(TypeNodeData::Any(TypeAny {
                    span: Span::new(token.start, token.end),
                })))
            }
            TokenKind::Question => self.parse_nullable_prefix(lexer, mode, disallow_conditional),
            TokenKind::Bang => self.parse_not_nullable_prefix(lexer, mode, disallow_conditional),
            TokenKind::Eq if mode.is_jsdoc() || mode.is_closure() => {
                self.parse_optional_prefix(lexer, mode, disallow_conditional)
            }
            TokenKind::Ellipsis => self.parse_variadic_prefix(lexer, mode, disallow_conditional),
            TokenKind::LParen => {
                self.parse_parenthesis_or_function(lexer, mode, disallow_conditional)
            }
            TokenKind::LBracket if mode.is_typescript() => {
                self.parse_tuple(lexer, mode, disallow_conditional)
            }
            TokenKind::LBrace => self.parse_object_type(lexer, mode, disallow_conditional),
            TokenKind::Function => self.parse_function_type(lexer, mode, disallow_conditional),
            TokenKind::Typeof if mode.is_typescript() || mode.is_closure() => {
                self.parse_typeof(lexer, mode, disallow_conditional)
            }
            TokenKind::Keyof if mode.is_typescript() => {
                self.parse_keyof(lexer, mode, disallow_conditional)
            }
            TokenKind::Readonly if mode.is_typescript() => {
                self.parse_readonly_array(lexer, mode, disallow_conditional)
            }
            TokenKind::Import if mode.is_typescript() => {
                self.parse_import_type(lexer, mode, disallow_conditional)
            }
            TokenKind::Infer if mode.is_typescript() => self.parse_infer(lexer, mode),
            TokenKind::Asserts if mode.is_typescript() => {
                self.parse_asserts(lexer, mode, disallow_conditional)
            }
            TokenKind::Unique if mode.is_typescript() => self.parse_unique_symbol(lexer),
            TokenKind::Number => self.parse_number_literal(lexer),
            TokenKind::StringValue => self.parse_string_literal(lexer),
            TokenKind::TemplateLiteral if mode.is_typescript() => {
                self.parse_template_literal(lexer, mode, disallow_conditional)
            }
            TokenKind::Module => {
                self.parse_special_name_path_or_name(lexer, mode, SpecialPathType::Module)
            }
            TokenKind::Event if mode.is_jsdoc() => {
                self.parse_special_name_path_or_name(lexer, mode, SpecialPathType::Event)
            }
            TokenKind::External if mode.is_jsdoc() => {
                self.parse_special_name_path_or_name(lexer, mode, SpecialPathType::External)
            }
            TokenKind::Symbol if mode.is_jsdoc() || mode.is_closure() => {
                self.parse_name(lexer, mode)
            }
            TokenKind::Extends
            | TokenKind::Is
            | TokenKind::In
            | TokenKind::Readonly
            | TokenKind::Event
            | TokenKind::External => self.parse_name(lexer, mode),
            _ => {
                self.type_diag(TypeDiagnosticKind::NoParsletFound);
                None
            }
        }
    }

    #[inline]
    fn cur_infix_precedence(&self, lexer: &Lexer<'a>, mode: ParseMode) -> Precedence {
        match lexer.current.kind {
            TokenKind::Pipe => Precedence::Union,
            TokenKind::Amp if mode.is_typescript() => Precedence::Intersection,
            TokenKind::Question => Precedence::Nullable,
            TokenKind::Bang => Precedence::Nullable,
            TokenKind::Eq => Precedence::Optional,
            TokenKind::LBracket => Precedence::ArrayBrackets,
            TokenKind::Lt => Precedence::Generic,
            TokenKind::Dot if lexer.next.kind == TokenKind::Lt => Precedence::Generic,
            TokenKind::Dot | TokenKind::Hash | TokenKind::Tilde => Precedence::NamePath,
            TokenKind::Arrow => Precedence::Arrow,
            TokenKind::Is if mode.is_typescript() => Precedence::Infix,
            TokenKind::Extends if mode.is_typescript() => Precedence::Infix,
            TokenKind::Ellipsis if mode.is_jsdoc() => Precedence::Infix,
            TokenKind::LParen if mode.is_jsdoc() || mode.is_closure() => Precedence::Symbol,
            _ => Precedence::All,
        }
    }

    fn parse_infix_type(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        match lexer.current.kind {
            TokenKind::Pipe => self.parse_union(lexer, mode, disallow_conditional, left),
            TokenKind::Amp if mode.is_typescript() => {
                self.parse_intersection(lexer, mode, disallow_conditional, left)
            }
            TokenKind::Lt => self.parse_generic(lexer, mode, disallow_conditional, left),
            TokenKind::Dot if lexer.next.kind == TokenKind::Lt => {
                self.parse_generic(lexer, mode, disallow_conditional, left)
            }
            TokenKind::LBracket => {
                self.parse_array_brackets_or_indexed(lexer, mode, disallow_conditional, left)
            }
            TokenKind::Dot | TokenKind::Hash | TokenKind::Tilde => {
                self.parse_name_path(lexer, mode, left)
            }
            TokenKind::Question => self.parse_nullable_suffix(lexer, left),
            TokenKind::Bang => self.parse_not_nullable_suffix(lexer, left),
            TokenKind::Eq => self.parse_optional_suffix(lexer, left),
            TokenKind::Arrow => self.parse_arrow_function(lexer, mode, disallow_conditional, left),
            TokenKind::Is if mode.is_typescript() => {
                self.parse_predicate(lexer, mode, disallow_conditional, left)
            }
            TokenKind::Extends if mode.is_typescript() => {
                self.parse_conditional(lexer, mode, disallow_conditional, left)
            }
            TokenKind::Ellipsis if mode.is_jsdoc() => self.parse_variadic_suffix(lexer, left),
            TokenKind::LParen if mode.is_jsdoc() || mode.is_closure() => {
                self.parse_symbol(lexer, mode, disallow_conditional, left)
            }
            _ => Some(left),
        }
    }

    fn try_parse_type<F, T>(&mut self, lexer: &mut Lexer<'a>, f: F) -> Option<T>
    where
        F: FnOnce(&mut Self, &mut Lexer<'a>) -> Option<T>,
    {
        let saved_lexer = lexer.save();
        let saved_diag_len = self.diagnostics.len();
        let result = f(self, lexer);
        if result.is_none() {
            lexer.restore(saved_lexer);
            self.diagnostics.truncate(saved_diag_len);
        }
        result
    }

    #[inline]
    fn eat(&self, lexer: &mut Lexer<'a>, kind: TokenKind) -> bool {
        if lexer.current.kind == kind {
            lexer.bump();
            true
        } else {
            false
        }
    }

    #[inline]
    fn expect(&mut self, lexer: &mut Lexer<'a>, kind: TokenKind) -> bool {
        if lexer.current.kind == kind {
            lexer.bump();
            true
        } else {
            self.type_diag(TypeDiagnosticKind::ExpectedToken);
            false
        }
    }

    #[inline]
    fn parse_name(
        &mut self,
        lexer: &mut Lexer<'a>,
        _mode: ParseMode,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let token = lexer.current;
        let text = lexer.token_text(token);
        lexer.bump();
        Some(Box::new(TypeNodeData::Name(TypeName {
            span: Span::new(token.start, token.end),
            value: text,
        })))
    }

    fn parse_nullable_prefix(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        match lexer.current.kind {
            TokenKind::EOF
            | TokenKind::Pipe
            | TokenKind::Comma
            | TokenKind::RParen
            | TokenKind::RBracket
            | TokenKind::RBrace
            | TokenKind::Gt
            | TokenKind::Eq => {
                return Some(Box::new(TypeNodeData::Unknown(TypeUnknown {
                    span: Span::new(start, start + 1),
                })));
            }
            _ => {}
        }
        let element =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Nullable)?;
        let end = element.span().end;
        Some(Box::new(TypeNodeData::Nullable(TypeNullable {
            span: Span::new(start, end),
            element,
            position: ModifierPosition::Prefix,
        })))
    }

    fn parse_not_nullable_prefix(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        let element =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Prefix)?;
        let end = element.span().end;
        Some(Box::new(TypeNodeData::NotNullable(TypeNotNullable {
            span: Span::new(start, end),
            element,
            position: ModifierPosition::Prefix,
        })))
    }

    fn parse_optional_prefix(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        let element =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Optional)?;
        let end = element.span().end;
        Some(Box::new(TypeNodeData::Optional(TypeOptional {
            span: Span::new(start, end),
            element,
            position: ModifierPosition::Prefix,
        })))
    }

    fn parse_variadic_prefix(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        if matches!(
            lexer.current.kind,
            TokenKind::EOF | TokenKind::Comma | TokenKind::RParen | TokenKind::RBracket
        ) {
            return Some(Box::new(TypeNodeData::Variadic(TypeVariadic {
                span: Span::new(start, start + 3),
                element: None,
                position: None,
                square_brackets: false,
            })));
        }
        if mode.is_jsdoc() && lexer.current.kind == TokenKind::LBracket {
            lexer.bump();
            let element =
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::All)?;
            if !self.expect(lexer, TokenKind::RBracket) {
                return None;
            }
            let end = lexer.current.start;
            return Some(Box::new(TypeNodeData::Variadic(TypeVariadic {
                span: Span::new(start, end),
                element: Some(element),
                position: Some(VariadicPosition::Prefix),
                square_brackets: true,
            })));
        }
        let element =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Prefix)?;
        let end = element.span().end;
        Some(Box::new(TypeNodeData::Variadic(TypeVariadic {
            span: Span::new(start, end),
            element: Some(element),
            position: Some(VariadicPosition::Prefix),
            square_brackets: false,
        })))
    }

    fn parse_parenthesis_or_function(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        if mode.is_typescript() {
            if let Some(result) = self.try_parse_type(lexer, |this, lex| {
                this.try_parse_arrow_function_prefix(lex, mode, disallow_conditional, start)
            }) {
                return Some(result);
            }
        }
        lexer.bump();
        let element = self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::All)?;
        if !self.expect(lexer, TokenKind::RParen) {
            return None;
        }
        let end = lexer.current.start;
        Some(Box::new(TypeNodeData::Parenthesis(TypeParenthesis {
            span: Span::new(start, end),
            element,
        })))
    }

    fn try_parse_arrow_function_prefix(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        start: u32,
    ) -> Option<Box<TypeNodeData<'a>>> {
        lexer.bump();
        let mut parameters = Vec::new();
        if lexer.current.kind != TokenKind::RParen {
            loop {
                let param = self.parse_key_value_or_type(lexer, mode, disallow_conditional)?;
                parameters.push(param);
                if !self.eat(lexer, TokenKind::Comma) {
                    break;
                }
                if lexer.current.kind == TokenKind::RParen {
                    break;
                }
            }
        }
        if !self.eat(lexer, TokenKind::RParen) {
            return None;
        }
        if lexer.current.kind != TokenKind::Arrow {
            return None;
        }
        lexer.bump();
        let return_type =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::All)?;
        let end = return_type.span().end;
        Some(Box::new(TypeNodeData::Function(TypeFunction {
            span: Span::new(start, end),
            parameters,
            return_type: Some(return_type),
            type_parameters: Vec::new(),
            constructor: false,
            arrow: true,
            parenthesis: true,
        })))
    }

    fn parse_key_value_or_type(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        if (lexer.current.kind == TokenKind::Identifier
            || lexer.current.kind == TokenKind::This
            || lexer.current.kind == TokenKind::New
            || lexer.current.kind.is_keyword())
            && (lexer.next.kind == TokenKind::Colon || lexer.next.kind == TokenKind::Question)
        {
            return self
                .try_parse_type(lexer, |this, lex| {
                    this.parse_key_value(lex, mode, disallow_conditional)
                })
                .or_else(|| {
                    self.parse_type_pratt(
                        lexer,
                        mode,
                        disallow_conditional,
                        Precedence::ParameterList,
                    )
                });
        }
        if lexer.current.kind == TokenKind::Ellipsis {
            let start = lexer.current.start;
            lexer.bump();
            if (lexer.current.kind == TokenKind::Identifier || lexer.current.kind.is_keyword())
                && lexer.next.kind == TokenKind::Colon
            {
                let key_token = lexer.current;
                let key = lexer.token_text(key_token);
                lexer.bump();
                lexer.bump();
                let right = self.parse_type_pratt(
                    lexer,
                    mode,
                    disallow_conditional,
                    Precedence::ParameterList,
                )?;
                let end = right.span().end;
                return Some(Box::new(TypeNodeData::KeyValue(TypeKeyValue {
                    span: Span::new(start, end),
                    key,
                    right: Some(right),
                    optional: false,
                    variadic: true,
                })));
            }
            let element =
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Prefix)?;
            let end = element.span().end;
            return Some(Box::new(TypeNodeData::Variadic(TypeVariadic {
                span: Span::new(start, end),
                element: Some(element),
                position: Some(VariadicPosition::Prefix),
                square_brackets: false,
            })));
        }
        self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::ParameterList)
    }

    fn parse_key_value(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        let key_token = lexer.current;
        let key = lexer.token_text(key_token);
        lexer.bump();
        let optional = self.eat(lexer, TokenKind::Question);
        if !self.eat(lexer, TokenKind::Colon) {
            return None;
        }
        let right =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::KeyValue)?;
        let end = right.span().end;
        Some(Box::new(TypeNodeData::KeyValue(TypeKeyValue {
            span: Span::new(start, end),
            key,
            right: Some(right),
            optional,
            variadic: false,
        })))
    }

    fn parse_tuple(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        let mut elements = Vec::new();
        if lexer.current.kind != TokenKind::RBracket {
            loop {
                let element = self.parse_key_value_or_type(lexer, mode, disallow_conditional)?;
                elements.push(element);
                if !self.eat(lexer, TokenKind::Comma) {
                    break;
                }
                if lexer.current.kind == TokenKind::RBracket {
                    break;
                }
            }
        }
        if !self.expect(lexer, TokenKind::RBracket) {
            self.type_diag(TypeDiagnosticKind::UnclosedTuple);
            return None;
        }
        let end = lexer.current.start;
        Some(Box::new(TypeNodeData::Tuple(TypeTuple { span: Span::new(start, end), elements })))
    }

    fn parse_object_type(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        let mut elements = Vec::new();
        let mut separator = None;
        if lexer.current.kind != TokenKind::RBrace {
            loop {
                let field = self.parse_object_field(lexer, mode, disallow_conditional)?;
                elements.push(field);
                if self.eat(lexer, TokenKind::Comma) {
                    separator = Some(ObjectSeparator::Comma);
                } else if self.eat(lexer, TokenKind::Semicolon) {
                    separator = Some(ObjectSeparator::Semicolon);
                } else {
                    break;
                }
            }
        }
        if !self.expect(lexer, TokenKind::RBrace) {
            self.type_diag(TypeDiagnosticKind::UnclosedObject);
            return None;
        }
        let end = lexer.current.start;
        Some(Box::new(TypeNodeData::Object(TypeObject {
            span: Span::new(start, end),
            elements,
            separator,
        })))
    }

    fn parse_object_field(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        let readonly = lexer.current.kind == TokenKind::Readonly
            && lexer.next.kind != TokenKind::Colon
            && lexer.next.kind != TokenKind::Comma
            && lexer.next.kind != TokenKind::Semicolon
            && lexer.next.kind != TokenKind::RBrace;
        if readonly {
            lexer.bump();
        }
        if lexer.current.kind == TokenKind::LBracket {
            return self.parse_index_signature_or_mapped(
                lexer,
                mode,
                disallow_conditional,
                start,
                readonly,
            );
        }
        if lexer.current.kind == TokenKind::LParen && !readonly {
            return self.parse_call_signature(lexer, mode, disallow_conditional, start);
        }
        if lexer.current.kind == TokenKind::New
            && (lexer.next.kind == TokenKind::LParen || lexer.next.kind == TokenKind::Lt)
            && mode.is_typescript()
            && !readonly
        {
            return self.parse_constructor_signature(lexer, mode, disallow_conditional, start);
        }
        if lexer.current.kind == TokenKind::Lt && !readonly {
            return self.parse_call_signature_with_type_params(
                lexer,
                mode,
                disallow_conditional,
                start,
            );
        }
        let quote = match lexer.current.kind {
            TokenKind::StringValue => {
                if lexer.token_text(lexer.current).starts_with('"') {
                    Some(QuoteStyle::Double)
                } else {
                    Some(QuoteStyle::Single)
                }
            }
            _ => None,
        };
        if mode.is_jsdoc()
            && !matches!(
                lexer.next.kind,
                TokenKind::Colon
                    | TokenKind::Question
                    | TokenKind::Comma
                    | TokenKind::Semicolon
                    | TokenKind::RBrace
            )
        {
            let left =
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::KeyValue)?;
            if self.eat(lexer, TokenKind::Colon) {
                let right =
                    self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::KeyValue)?;
                let end = right.span().end;
                return Some(Box::new(TypeNodeData::JsdocObjectField(TypeJsdocObjectField {
                    span: Span::new(start, end),
                    left,
                    right,
                })));
            }
            let end = left.span().end;
            return Some(Box::new(TypeNodeData::JsdocObjectField(TypeJsdocObjectField {
                span: Span::new(start, end),
                left: Box::new(TypeNodeData::Name(TypeName {
                    span: Span::new(start, start),
                    value: "",
                })),
                right: left,
            })));
        }
        let key_token = lexer.current;
        let key_text = lexer.token_text(key_token);
        lexer.bump();
        if (lexer.current.kind == TokenKind::LParen || lexer.current.kind == TokenKind::Lt)
            && mode.is_typescript()
            && !readonly
        {
            return self.parse_method_signature(
                lexer,
                mode,
                disallow_conditional,
                start,
                key_text,
                quote,
            );
        }
        let key = Box::new(TypeNodeData::Name(TypeName {
            span: Span::new(key_token.start, key_token.end),
            value: key_text,
        }));
        let optional = self.eat(lexer, TokenKind::Question);
        let right = if self.eat(lexer, TokenKind::Colon) {
            Some(self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::KeyValue)?)
        } else {
            None
        };
        let end = right.as_ref().map_or(key_token.end, |r| r.span().end);
        Some(Box::new(TypeNodeData::ObjectField(TypeObjectField {
            span: Span::new(start, end),
            key,
            right,
            optional,
            readonly,
            quote,
        })))
    }

    fn parse_type_parameters(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Vec<Box<TypeNodeData<'a>>> {
        let mut type_params = Vec::new();
        if lexer.current.kind != TokenKind::Lt {
            return type_params;
        }
        lexer.bump();
        loop {
            let tp_start = lexer.current.start;
            let name_token = lexer.current;
            let name_text = lexer.token_text(name_token);
            lexer.bump();
            let name = Box::new(TypeNodeData::Name(TypeName {
                span: Span::new(name_token.start, name_token.end),
                value: name_text,
            }));
            let constraint = if lexer.current.kind == TokenKind::Extends {
                lexer.bump();
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Optional)
            } else {
                None
            };
            let default_value = if self.eat(lexer, TokenKind::Eq) {
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Optional)
            } else {
                None
            };
            let end = default_value
                .as_ref()
                .map_or(constraint.as_ref().map_or(name_token.end, |c| c.span().end), |d| {
                    d.span().end
                });
            type_params.push(Box::new(TypeNodeData::TypeParameter(TypeTypeParameter {
                span: Span::new(tp_start, end),
                name,
                constraint,
                default_value,
            })));
            if !self.eat(lexer, TokenKind::Comma) {
                break;
            }
        }
        self.expect(lexer, TokenKind::Gt);
        type_params
    }

    fn parse_call_signature(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        start: u32,
    ) -> Option<Box<TypeNodeData<'a>>> {
        lexer.bump();
        let mut parameters = Vec::new();
        if lexer.current.kind != TokenKind::RParen {
            loop {
                let param = self.parse_key_value_or_type(lexer, mode, disallow_conditional)?;
                parameters.push(param);
                if !self.eat(lexer, TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(lexer, TokenKind::RParen);
        self.expect(lexer, TokenKind::Colon);
        let return_type =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Prefix)?;
        let end = return_type.span().end;
        Some(Box::new(TypeNodeData::CallSignature(TypeCallSignature {
            span: Span::new(start, end),
            parameters,
            return_type,
            type_parameters: Vec::new(),
        })))
    }

    fn parse_call_signature_with_type_params(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        start: u32,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let type_parameters = self.parse_type_parameters(lexer, mode, disallow_conditional);
        self.expect(lexer, TokenKind::LParen);
        let mut parameters = Vec::new();
        if lexer.current.kind != TokenKind::RParen {
            loop {
                let param = self.parse_key_value_or_type(lexer, mode, disallow_conditional)?;
                parameters.push(param);
                if !self.eat(lexer, TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(lexer, TokenKind::RParen);
        self.expect(lexer, TokenKind::Colon);
        let return_type =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Prefix)?;
        let end = return_type.span().end;
        Some(Box::new(TypeNodeData::CallSignature(TypeCallSignature {
            span: Span::new(start, end),
            parameters,
            return_type,
            type_parameters,
        })))
    }

    fn parse_constructor_signature(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        start: u32,
    ) -> Option<Box<TypeNodeData<'a>>> {
        lexer.bump();
        let type_parameters = self.parse_type_parameters(lexer, mode, disallow_conditional);
        self.expect(lexer, TokenKind::LParen);
        let mut parameters = Vec::new();
        if lexer.current.kind != TokenKind::RParen {
            loop {
                let param = self.parse_key_value_or_type(lexer, mode, disallow_conditional)?;
                parameters.push(param);
                if !self.eat(lexer, TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(lexer, TokenKind::RParen);
        self.expect(lexer, TokenKind::Colon);
        let return_type =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Prefix)?;
        let end = return_type.span().end;
        Some(Box::new(TypeNodeData::ConstructorSignature(TypeConstructorSignature {
            span: Span::new(start, end),
            parameters,
            return_type,
            type_parameters,
        })))
    }

    #[allow(clippy::too_many_arguments)]
    fn parse_method_signature(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        start: u32,
        name: &'a str,
        quote: Option<QuoteStyle>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let type_parameters = self.parse_type_parameters(lexer, mode, disallow_conditional);
        self.expect(lexer, TokenKind::LParen);
        let mut parameters = Vec::new();
        if lexer.current.kind != TokenKind::RParen {
            loop {
                let param = self.parse_key_value_or_type(lexer, mode, disallow_conditional)?;
                parameters.push(param);
                if !self.eat(lexer, TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(lexer, TokenKind::RParen);
        self.expect(lexer, TokenKind::Colon);
        let return_type =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Prefix)?;
        let end = return_type.span().end;
        Some(Box::new(TypeNodeData::MethodSignature(TypeMethodSignature {
            span: Span::new(start, end),
            name,
            parameters,
            return_type,
            type_parameters,
            quote,
        })))
    }

    fn parse_index_signature_or_mapped(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        start: u32,
        readonly: bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        lexer.bump();
        let key_token = lexer.current;
        let key = lexer.token_text(key_token);
        lexer.bump();
        if lexer.current.kind == TokenKind::In {
            lexer.bump();
            let _right =
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::All)?;
            self.expect(lexer, TokenKind::RBracket);
            self.eat(lexer, TokenKind::Question);
            self.expect(lexer, TokenKind::Colon);
            let value =
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::KeyValue)?;
            let end = value.span().end;
            return Some(Box::new(TypeNodeData::MappedType(TypeMappedType {
                span: Span::new(start, end),
                key,
                right: value,
            })));
        }
        if lexer.current.kind == TokenKind::Colon {
            lexer.bump();
            let _index_type =
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::All)?;
            self.expect(lexer, TokenKind::RBracket);
            self.expect(lexer, TokenKind::Colon);
            let value =
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::KeyValue)?;
            let end = value.span().end;
            return Some(Box::new(TypeNodeData::IndexSignature(TypeIndexSignature {
                span: Span::new(start, end),
                key,
                right: value,
            })));
        }
        self.expect(lexer, TokenKind::RBracket);
        let optional = self.eat(lexer, TokenKind::Question);
        if lexer.current.kind == TokenKind::LParen || lexer.current.kind == TokenKind::Lt {
            let type_parameters = self.parse_type_parameters(lexer, mode, disallow_conditional);
            self.expect(lexer, TokenKind::LParen);
            let mut parameters = Vec::new();
            if lexer.current.kind != TokenKind::RParen {
                loop {
                    let param = self.parse_key_value_or_type(lexer, mode, disallow_conditional)?;
                    parameters.push(param);
                    if !self.eat(lexer, TokenKind::Comma) {
                        break;
                    }
                    if lexer.current.kind == TokenKind::RParen {
                        break;
                    }
                }
            }
            self.expect(lexer, TokenKind::RParen);
            self.expect(lexer, TokenKind::Colon);
            let return_type =
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Prefix)?;
            let end = return_type.span().end;
            let key_node = Box::new(TypeNodeData::Name(TypeName {
                span: Span::new(key_token.start, key_token.end),
                value: key,
            }));
            return Some(Box::new(TypeNodeData::ObjectField(TypeObjectField {
                span: Span::new(start, end),
                key: key_node,
                right: Some(Box::new(TypeNodeData::Function(TypeFunction {
                    span: Span::new(key_token.start, end),
                    parameters,
                    return_type: Some(return_type),
                    type_parameters,
                    constructor: false,
                    arrow: false,
                    parenthesis: true,
                }))),
                optional,
                readonly,
                quote: None,
            })));
        }
        self.expect(lexer, TokenKind::Colon);
        let value =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::KeyValue)?;
        let end = value.span().end;
        let key_node = Box::new(TypeNodeData::Name(TypeName {
            span: Span::new(key_token.start, key_token.end),
            value: key,
        }));
        Some(Box::new(TypeNodeData::ObjectField(TypeObjectField {
            span: Span::new(start, end),
            key: key_node,
            right: Some(value),
            optional,
            readonly,
            quote: None,
        })))
    }

    fn parse_function_type(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        if lexer.current.kind != TokenKind::LParen {
            if mode.is_jsdoc() {
                return Some(Box::new(TypeNodeData::Function(TypeFunction {
                    span: Span::new(start, start + 8),
                    parameters: Vec::new(),
                    return_type: None,
                    type_parameters: Vec::new(),
                    constructor: false,
                    arrow: false,
                    parenthesis: false,
                })));
            }
            return Some(Box::new(TypeNodeData::Name(TypeName {
                span: Span::new(start, start + 8),
                value: "function",
            })));
        }
        lexer.bump();
        let mut parameters = Vec::new();
        let mut constructor = false;
        if lexer.current.kind != TokenKind::RParen {
            if lexer.current.kind == TokenKind::New && lexer.next.kind == TokenKind::Colon {
                constructor = true;
            }
            loop {
                let param = self.parse_key_value_or_type(lexer, mode, disallow_conditional)?;
                parameters.push(param);
                if !self.eat(lexer, TokenKind::Comma) {
                    break;
                }
                if lexer.current.kind == TokenKind::RParen {
                    break;
                }
            }
        }
        self.expect(lexer, TokenKind::RParen);
        let return_type = if self.eat(lexer, TokenKind::Colon) {
            Some(self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Prefix)?)
        } else {
            None
        };
        let end = return_type.as_ref().map_or(lexer.current.start, |r| r.span().end);
        Some(Box::new(TypeNodeData::Function(TypeFunction {
            span: Span::new(start, end),
            parameters,
            return_type,
            type_parameters: Vec::new(),
            constructor,
            arrow: false,
            parenthesis: true,
        })))
    }

    fn parse_new_function(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        lexer.bump();
        let mut parameters = Vec::new();
        if lexer.current.kind != TokenKind::RParen {
            loop {
                let param = self.parse_key_value_or_type(lexer, mode, disallow_conditional)?;
                parameters.push(param);
                if !self.eat(lexer, TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(lexer, TokenKind::RParen);
        let (return_type, arrow) = if self.eat(lexer, TokenKind::Arrow) {
            (
                Some(self.parse_type_pratt(
                    lexer,
                    mode,
                    disallow_conditional,
                    Precedence::Prefix,
                )?),
                true,
            )
        } else if self.eat(lexer, TokenKind::Colon) {
            (
                Some(self.parse_type_pratt(
                    lexer,
                    mode,
                    disallow_conditional,
                    Precedence::Prefix,
                )?),
                false,
            )
        } else {
            (None, false)
        };
        let end = return_type.as_ref().map_or(lexer.current.start, |r| r.span().end);
        Some(Box::new(TypeNodeData::Function(TypeFunction {
            span: Span::new(start, end),
            parameters,
            return_type,
            type_parameters: Vec::new(),
            constructor: true,
            arrow,
            parenthesis: true,
        })))
    }

    fn parse_typeof(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        let element =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::KeyOfTypeOf)?;
        let end = element.span().end;
        Some(Box::new(TypeNodeData::TypeOf(TypeTypeOf { span: Span::new(start, end), element })))
    }

    fn parse_keyof(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        let element =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::KeyOfTypeOf)?;
        let end = element.span().end;
        Some(Box::new(TypeNodeData::KeyOf(TypeKeyOf { span: Span::new(start, end), element })))
    }

    fn parse_readonly_array(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        let element =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Prefix)?;
        let end = element.span().end;
        Some(Box::new(TypeNodeData::ReadonlyArray(TypeReadonlyArray {
            span: Span::new(start, end),
            element,
        })))
    }

    fn parse_import_type(
        &mut self,
        lexer: &mut Lexer<'a>,
        _mode: ParseMode,
        _disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        self.expect(lexer, TokenKind::LParen);
        let element_token = lexer.current;
        if element_token.kind != TokenKind::StringValue {
            self.type_diag(TypeDiagnosticKind::ExpectedToken);
            return None;
        }
        let text = lexer.token_text(element_token);
        let element = Box::new(TypeNodeData::StringValue(TypeStringValue {
            span: Span::new(element_token.start, element_token.end),
            value: text,
            quote: if text.starts_with('"') { QuoteStyle::Double } else { QuoteStyle::Single },
        }));
        lexer.bump();
        if !self.expect(lexer, TokenKind::RParen) {
            return None;
        }
        let end = lexer.current.start;
        Some(Box::new(TypeNodeData::Import(TypeImport { span: Span::new(start, end), element })))
    }

    fn parse_infer(
        &mut self,
        lexer: &mut Lexer<'a>,
        _mode: ParseMode,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        let name_token = lexer.current;
        let name_text = lexer.token_text(name_token);
        lexer.bump();
        let element = Box::new(TypeNodeData::Name(TypeName {
            span: Span::new(name_token.start, name_token.end),
            value: name_text,
        }));
        let end = name_token.end;
        Some(Box::new(TypeNodeData::Infer(TypeInfer { span: Span::new(start, end), element })))
    }

    fn parse_asserts(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        if !matches!(lexer.current.kind, TokenKind::Identifier | TokenKind::This | TokenKind::New)
            && !lexer.current.kind.is_keyword()
        {
            self.type_diag(TypeDiagnosticKind::ExpectedToken);
            return None;
        }
        let name_token = lexer.current;
        let name_text = lexer.token_text(name_token);
        lexer.bump();
        let left = Box::new(TypeNodeData::Name(TypeName {
            span: Span::new(name_token.start, name_token.end),
            value: name_text,
        }));
        if lexer.current.kind == TokenKind::Is {
            lexer.bump();
            let right =
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::All)?;
            let end = right.span().end;
            Some(Box::new(TypeNodeData::Asserts(TypeAsserts {
                span: Span::new(start, end),
                left,
                right,
            })))
        } else {
            let end = name_token.end;
            Some(Box::new(TypeNodeData::AssertsPlain(TypeAssertsPlain {
                span: Span::new(start, end),
                element: left,
            })))
        }
    }

    fn parse_unique_symbol(&mut self, lexer: &mut Lexer<'a>) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        lexer.bump();
        if lexer.current.kind == TokenKind::Symbol {
            let end = lexer.current.end;
            lexer.bump();
            Some(Box::new(TypeNodeData::UniqueSymbol(TypeUniqueSymbol {
                span: Span::new(start, end),
            })))
        } else {
            Some(Box::new(TypeNodeData::Name(TypeName {
                span: Span::new(start, start + 6),
                value: "unique",
            })))
        }
    }

    fn parse_number_literal(&mut self, lexer: &mut Lexer<'a>) -> Option<Box<TypeNodeData<'a>>> {
        let token = lexer.current;
        let text = lexer.token_text(token);
        lexer.bump();
        Some(Box::new(TypeNodeData::Number(TypeNumber {
            span: Span::new(token.start, token.end),
            value: text,
        })))
    }

    fn parse_string_literal(&mut self, lexer: &mut Lexer<'a>) -> Option<Box<TypeNodeData<'a>>> {
        let token = lexer.current;
        let text = lexer.token_text(token);
        let quote = if text.starts_with('"') { QuoteStyle::Double } else { QuoteStyle::Single };
        lexer.bump();
        Some(Box::new(TypeNodeData::StringValue(TypeStringValue {
            span: Span::new(token.start, token.end),
            value: text,
            quote,
        })))
    }

    fn parse_template_literal(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let token = lexer.current;
        let text = lexer.token_text(token);
        lexer.bump();
        let mut literals: Vec<&'a str> = Vec::new();
        let mut interpolations: Vec<Box<TypeNodeData<'a>>> = Vec::new();
        let inner = if text.len() >= 2 { &text[1..text.len() - 1] } else { text };
        let bytes = inner.as_bytes();
        let mut pos = 0;
        let mut lit_start = 0;
        while pos < bytes.len() {
            if bytes[pos] == b'\\' && pos + 1 < bytes.len() {
                pos += 2;
            } else if bytes[pos] == b'$' && pos + 1 < bytes.len() && bytes[pos + 1] == b'{' {
                literals.push(&inner[lit_start..pos]);
                let interp_start = pos + 2;
                let mut depth = 1u32;
                let mut interp_end = interp_start;
                while interp_end < bytes.len() && depth > 0 {
                    if bytes[interp_end] == b'{' {
                        depth += 1;
                    } else if bytes[interp_end] == b'}' {
                        depth -= 1;
                    }
                    if depth > 0 {
                        interp_end += 1;
                    }
                }
                let interp_text = &inner[interp_start..interp_end];
                let interp_base = token.start + 1 + interp_start as u32;
                let mut interp_lexer = Lexer::new(interp_text, interp_base, mode.is_loose());
                if let Some(node) = self.parse_type_pratt(
                    &mut interp_lexer,
                    mode,
                    disallow_conditional,
                    Precedence::All,
                ) {
                    interpolations.push(node);
                }
                pos = if interp_end < bytes.len() { interp_end + 1 } else { interp_end };
                lit_start = pos;
            } else {
                pos += 1;
            }
        }
        literals.push(&inner[lit_start..]);
        Some(Box::new(TypeNodeData::TemplateLiteral(TypeTemplateLiteral {
            span: Span::new(token.start, token.end),
            literals,
            interpolations,
        })))
    }

    fn parse_special_name_path_or_name(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        special_type: SpecialPathType,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = lexer.current.start;
        if lexer.next.kind == TokenKind::Colon {
            lexer.bump();
            lexer.bump();
            let quote = match lexer.current.kind {
                TokenKind::StringValue => {
                    let text = lexer.token_text(lexer.current);
                    if text.starts_with('"') {
                        Some(QuoteStyle::Double)
                    } else {
                        Some(QuoteStyle::Single)
                    }
                }
                _ => None,
            };
            let value_start = lexer.current.start;
            let mut value_end = lexer.current.end;
            lexer.bump();
            while matches!(
                lexer.current.kind,
                TokenKind::Dot | TokenKind::Slash | TokenKind::Identifier
            ) || lexer.current.kind.is_keyword()
            {
                value_end = lexer.current.end;
                lexer.bump();
            }
            let raw_value = self.get_type_source_text(lexer, value_start, value_end);
            let value = if quote.is_some() && raw_value.len() >= 2 {
                &raw_value[1..raw_value.len() - 1]
            } else {
                raw_value
            };
            return Some(Box::new(TypeNodeData::SpecialNamePath(TypeSpecialNamePath {
                span: Span::new(start, value_end),
                value,
                special_type,
                quote,
            })));
        }
        self.parse_name(lexer, mode)
    }

    fn get_type_source_text(&self, lexer: &Lexer<'a>, abs_start: u32, abs_end: u32) -> &'a str {
        let token =
            super::token::Token::new(super::token::TokenKind::Identifier, abs_start, abs_end);
        lexer.token_text(token)
    }

    // Infix implementations -------------------------------------------------

    fn parse_union(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = left.span().start;
        lexer.bump();
        let mut elements = Vec::with_capacity(4);
        elements.push(left);
        loop {
            let element =
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Union)?;
            elements.push(element);
            if !self.eat(lexer, TokenKind::Pipe) {
                break;
            }
        }
        let end = elements.last().unwrap().span().end;
        Some(Box::new(TypeNodeData::Union(TypeUnion { span: Span::new(start, end), elements })))
    }

    fn parse_intersection(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = left.span().start;
        lexer.bump();
        let mut elements = Vec::with_capacity(4);
        elements.push(left);
        loop {
            let element =
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::Intersection)?;
            elements.push(element);
            if !self.eat(lexer, TokenKind::Amp) {
                break;
            }
        }
        let end = elements.last().unwrap().span().end;
        Some(Box::new(TypeNodeData::Intersection(TypeIntersection {
            span: Span::new(start, end),
            elements,
        })))
    }

    fn parse_generic(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = left.span().start;
        let dot = self.eat(lexer, TokenKind::Dot);
        lexer.bump();
        let mut elements = Vec::with_capacity(4);
        loop {
            let element = self.parse_type_pratt(
                lexer,
                mode,
                disallow_conditional,
                Precedence::ParameterList,
            )?;
            elements.push(element);
            if !self.eat(lexer, TokenKind::Comma) {
                break;
            }
        }
        if !self.expect(lexer, TokenKind::Gt) {
            self.type_diag(TypeDiagnosticKind::UnclosedGeneric);
            return None;
        }
        let end = lexer.current.start;
        Some(Box::new(TypeNodeData::Generic(TypeGeneric {
            span: Span::new(start, end),
            left,
            elements,
            brackets: GenericBrackets::Angle,
            dot,
        })))
    }

    fn parse_array_brackets_or_indexed(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = left.span().start;
        lexer.bump();
        if lexer.current.kind == TokenKind::RBracket {
            lexer.bump();
            let end = lexer.current.start;
            return Some(Box::new(TypeNodeData::Generic(TypeGeneric {
                span: Span::new(start, end),
                left,
                elements: Vec::new(),
                brackets: GenericBrackets::Square,
                dot: false,
            })));
        }
        if lexer.current.kind == TokenKind::StringValue {
            let prop_token = lexer.current;
            let prop_text = lexer.token_text(prop_token);
            let quote = if prop_text.starts_with('"') {
                Some(QuoteStyle::Double)
            } else {
                Some(QuoteStyle::Single)
            };
            let unquoted =
                if prop_text.len() >= 2 { &prop_text[1..prop_text.len() - 1] } else { prop_text };
            lexer.bump();
            self.expect(lexer, TokenKind::RBracket);
            let end = lexer.current.start;
            let right = Box::new(TypeNodeData::Property(TypeProperty {
                span: Span::new(prop_token.start, prop_token.end),
                value: unquoted,
                quote,
            }));
            return Some(Box::new(TypeNodeData::NamePath(TypeNamePath {
                span: Span::new(start, end),
                left,
                right,
                path_type: NamePathType::PropertyBrackets,
            })));
        }
        if mode.is_typescript() {
            let index =
                self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::All)?;
            if !self.expect(lexer, TokenKind::RBracket) {
                return None;
            }
            let end = lexer.current.start;
            let right = Box::new(TypeNodeData::IndexedAccessIndex(TypeIndexedAccessIndex {
                span: Span::new(start, end),
                right: index,
            }));
            return Some(Box::new(TypeNodeData::NamePath(TypeNamePath {
                span: Span::new(start, end),
                left,
                right,
                path_type: NamePathType::PropertyBrackets,
            })));
        }
        if !self.expect(lexer, TokenKind::RBracket) {
            return None;
        }
        let end = lexer.current.start;
        Some(Box::new(TypeNodeData::Generic(TypeGeneric {
            span: Span::new(start, end),
            left,
            elements: Vec::new(),
            brackets: GenericBrackets::Square,
            dot: false,
        })))
    }

    fn parse_name_path(
        &mut self,
        lexer: &mut Lexer<'a>,
        _mode: ParseMode,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = left.span().start;
        let path_type = match lexer.current.kind {
            TokenKind::Dot => NamePathType::Property,
            TokenKind::Hash => NamePathType::Instance,
            TokenKind::Tilde => NamePathType::Inner,
            _ => return Some(left),
        };
        lexer.bump();
        let right_token = lexer.current;
        let right_text = lexer.token_text(right_token);
        let quote = match right_token.kind {
            TokenKind::StringValue => {
                if right_text.starts_with('"') {
                    Some(QuoteStyle::Double)
                } else {
                    Some(QuoteStyle::Single)
                }
            }
            _ => None,
        };
        lexer.bump();
        let right = Box::new(TypeNodeData::Property(TypeProperty {
            span: Span::new(right_token.start, right_token.end),
            value: right_text,
            quote,
        }));
        let end = right_token.end;
        Some(Box::new(TypeNodeData::NamePath(TypeNamePath {
            span: Span::new(start, end),
            left,
            right,
            path_type,
        })))
    }

    fn parse_nullable_suffix(
        &mut self,
        lexer: &mut Lexer<'a>,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = left.span().start;
        let end = lexer.current.end;
        lexer.bump();
        Some(Box::new(TypeNodeData::Nullable(TypeNullable {
            span: Span::new(start, end),
            element: left,
            position: ModifierPosition::Suffix,
        })))
    }

    fn parse_not_nullable_suffix(
        &mut self,
        lexer: &mut Lexer<'a>,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = left.span().start;
        let end = lexer.current.end;
        lexer.bump();
        Some(Box::new(TypeNodeData::NotNullable(TypeNotNullable {
            span: Span::new(start, end),
            element: left,
            position: ModifierPosition::Suffix,
        })))
    }

    fn parse_optional_suffix(
        &mut self,
        lexer: &mut Lexer<'a>,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = left.span().start;
        let end = lexer.current.end;
        lexer.bump();
        Some(Box::new(TypeNodeData::Optional(TypeOptional {
            span: Span::new(start, end),
            element: left,
            position: ModifierPosition::Suffix,
        })))
    }

    fn parse_arrow_function(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = left.span().start;
        lexer.bump();
        let return_type =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::All)?;
        let end = return_type.span().end;
        let parameters = Vec::new();
        Some(Box::new(TypeNodeData::Function(TypeFunction {
            span: Span::new(start, end),
            parameters,
            return_type: Some(return_type),
            type_parameters: Vec::new(),
            constructor: false,
            arrow: true,
            parenthesis: true,
        })))
    }

    fn parse_predicate(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = left.span().start;
        lexer.bump();
        let right = self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::All)?;
        let end = right.span().end;
        Some(Box::new(TypeNodeData::Predicate(TypePredicate {
            span: Span::new(start, end),
            left,
            right,
        })))
    }

    fn parse_conditional(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        if *disallow_conditional {
            return Some(left);
        }
        let start = left.span().start;
        lexer.bump();
        let mut nested_disallow = true;
        let extends_type =
            self.parse_type_pratt(lexer, mode, &mut nested_disallow, Precedence::All)?;
        self.expect(lexer, TokenKind::Question);
        let true_type =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::All)?;
        self.expect(lexer, TokenKind::Colon);
        let false_type =
            self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::All)?;
        let end = false_type.span().end;
        Some(Box::new(TypeNodeData::Conditional(TypeConditional {
            span: Span::new(start, end),
            checks_type: left,
            extends_type,
            true_type,
            false_type,
        })))
    }

    fn parse_variadic_suffix(
        &mut self,
        lexer: &mut Lexer<'a>,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = left.span().start;
        let end = lexer.current.end;
        lexer.bump();
        Some(Box::new(TypeNodeData::Variadic(TypeVariadic {
            span: Span::new(start, end),
            element: Some(left),
            position: Some(VariadicPosition::Suffix),
            square_brackets: false,
        })))
    }

    fn parse_symbol(
        &mut self,
        lexer: &mut Lexer<'a>,
        mode: ParseMode,
        disallow_conditional: &mut bool,
        left: Box<TypeNodeData<'a>>,
    ) -> Option<Box<TypeNodeData<'a>>> {
        let start = left.span().start;
        let value = match left.as_ref() {
            TypeNodeData::Name(name) => name.value,
            _ => {
                self.type_diag(TypeDiagnosticKind::InvalidTypeExpression);
                return None;
            }
        };
        lexer.bump();
        let element = if lexer.current.kind != TokenKind::RParen {
            Some(self.parse_type_pratt(lexer, mode, disallow_conditional, Precedence::All)?)
        } else {
            None
        };
        if !self.expect(lexer, TokenKind::RParen) {
            return None;
        }
        let end = lexer.current.start;
        Some(Box::new(TypeNodeData::Symbol(TypeSymbol {
            span: Span::new(start, end),
            value,
            element,
        })))
    }
}
