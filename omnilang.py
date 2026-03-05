#!/usr/bin/env python3
"""
OmniLang - The Ultimate Programming Language
A simplified interpreter for demonstration purposes.
"""

import sys
import re
from typing import Any, Dict, List, Optional, Union

# Token types
TokenType = str
Token = tuple

class Lexer:
    """Lexical analyzer for OmniLang"""
    
    KEYWORDS = {
        'fn', 'let', 'mut', 'if', 'else', 'match', 'loop', 'while', 'for', 'in',
        'return', 'break', 'continue', 'struct', 'enum', 'impl', 'pub', 'async',
        'await', 'try', 'catch', 'throw', 'true', 'false', 'null', 'and', 'or',
        'self', 'super', 'where', 'unsafe', 'type', 'as', 'use', 'mod', 'trait'
    }
    
    def __init__(self, source: str):
        self.source = source
        self.pos = 0
        self.line = 1
        self.column = 1
        self.tokens: List[Token] = []
    
    def current_char(self) -> Optional[str]:
        if self.pos < len(self.source):
            return self.source[self.pos]
        return None
    
    def peek(self, offset: int = 1) -> Optional[str]:
        if self.pos + offset < len(self.source):
            return self.source[self.pos + offset]
        return None
    
    def advance(self) -> Optional[str]:
        if self.pos < len(self.source):
            c = self.source[self.pos]
            self.pos += 1
            if c == '\n':
                self.line += 1
                self.column = 1
            else:
                self.column += 1
            return c
        return None
    
    def skip_whitespace(self):
        while self.current_char() and self.current_char() in ' \t\r\n':
            self.advance()
    
    def skip_comment(self):
        # Handle both # and // style comments
        if self.current_char() == '#':
            while self.current_char() and self.current_char() != '\n':
                self.advance()
        elif self.current_char() == '/' and self.peek() == '/':
            while self.current_char() and self.current_char() != '\n':
                self.advance()
    
    def read_string(self) -> str:
        quote = self.advance()
        string = ""
        while self.current_char() and self.current_char() != quote:
            if self.current_char() == '\\':
                self.advance()
                escape = self.current_char()
                string += {'n': '\n', 't': '\t', 'r': '\r', '"': '"', "'": "'", '\\': '\\'}.get(escape, escape)
            else:
                string += self.current_char()
            self.advance()
        self.advance()  # closing quote
        return string
    
    def read_number(self) -> Union[int, float]:
        num_str = ""
        is_float = False
        while self.current_char() and (self.current_char().isdigit() or self.current_char() == '.'):
            if self.current_char() == '.':
                is_float = True
            num_str += self.advance()
        if is_float:
            return float(num_str)
        return int(num_str)
    
    def read_identifier(self) -> str:
        ident = ""
        while self.current_char() and (self.current_char().isalnum() or self.current_char() == '_'):
            ident += self.advance()
        return ident
    
    def tokenize(self) -> List[Token]:
        while self.pos < len(self.source):
            self.skip_whitespace()
            self.skip_comment()
            
            if not self.current_char():
                break
            
            c = self.current_char()
            
            # String
            if c in '"\'':
                self.tokens.append(('STRING', self.read_string()))
                continue
            
            # Number
            if c.isdigit():
                # Check for range operator
                num_str = ""
                while self.current_char() and self.current_char().isdigit():
                    num_str += self.advance()
                
                # Check for range
                if self.current_char() == '.':
                    self.advance()
                    if self.current_char() == '.':
                        self.advance()
                        self.tokens.append(('NUMBER', int(num_str)))
                        self.tokens.append(('RANGE', '..'))
                        continue
                    else:
                        # It's a float
                        num_str += '.' + self.advance() + self.read_number()
                        self.tokens.append(('NUMBER', float(num_str)))
                        continue
                
                self.tokens.append(('NUMBER', int(num_str)))
                continue
            
            # Identifier/keyword
            if c.isalpha() or c == '_':
                ident = self.read_identifier()
                if ident in self.KEYWORDS:
                    self.tokens.append(('KEYWORD', ident))
                else:
                    self.tokens.append(('IDENT', ident))
                continue
            
            # Operators and punctuation
            op_map = {
                '+': 'PLUS', '-': 'MINUS', '*': 'STAR', '/': 'SLASH',
                '%': 'PERCENT', '^': 'CARET', '&': 'AMP', '|': 'PIPE',
                '!': 'BANG', '=': 'EQ', '<': 'LT', '>': 'GT',
                '(': 'LPAREN', ')': 'RPAREN', '[': 'LBRACKET', ']': 'RBRACKET',
                '{': 'LBRACE', '}': 'RBRACE', ',': 'COMMA', '.': 'DOT',
                ':': 'COLON', ';': 'SEMI', '?': 'QUESTION'
            }
            
            if c in op_map:
                # Check for multi-character operators
                self.advance()
                # Handle // as comment
                if c == '/' and self.current_char() == '/':
                    while self.current_char() and self.current_char() != '\n':
                        self.advance()
                    continue
                elif c == '-' and self.current_char() == '>':
                    self.advance()  # consume the >
                    self.tokens.append(('ARROW', '->'))
                elif c == '=' and self.current_char() == '=':
                    self.advance()
                    self.tokens.append(('COMPARE', '=='))
                elif c == '!' and self.current_char() == '=':
                    self.advance()
                    self.tokens.append(('COMPARE', '!='))
                elif c == '<' and self.current_char() == '=':
                    self.advance()
                    self.tokens.append(('COMPARE', '<='))
                elif c == '>' and self.current_char() == '=':
                    self.advance()
                    self.tokens.append(('COMPARE', '>='))
                elif c == '&' and self.current_char() == '&':
                    self.advance()
                    self.tokens.append(('LOGICAL', '&&'))
                elif c == '|' and self.current_char() == '|':
                    self.advance()
                    self.tokens.append(('LOGICAL', '||'))
                else:
                    self.tokens.append((op_map[c], c))
                continue
            
            # Unknown character
            print(f"Warning: Unknown character '{c}' at line {self.line}, column {self.column}")
            self.advance()
        
        self.tokens.append(('EOF', None))
        return self.tokens


class Parser:
    """Parser for OmniLang - creates AST"""
    
    def __init__(self, tokens: List[Token]):
        self.tokens = tokens
        self.pos = 0
    
    def current(self) -> Token:
        if self.pos < len(self.tokens):
            return self.tokens[self.pos]
        return ('EOF', None)
    
    def advance(self) -> Token:
        token = self.current()
        if self.pos < len(self.tokens):
            self.pos += 1
        return token
    
    def expect(self, token_type: str) -> Token:
        if self.current()[0] == token_type:
            return self.advance()
        raise SyntaxError(f"Expected {token_type}, got {self.current()}")
    
    def parse(self) -> List[Dict]:
        statements = []
        while self.current()[0] != 'EOF':
            statements.append(self.parse_statement())
        return statements
    
    def parse_statement(self) -> Dict:
        token = self.current()
        
        if token[0] == 'KEYWORD':
            if token[1] == 'fn':
                return self.parse_function()
            elif token[1] in ('let', 'mut'):
                return self.parse_variable_decl()
            elif token[1] == 'if':
                return self.parse_if()
            elif token[1] == 'match':
                return self.parse_match()
            elif token[1] == 'return':
                return self.parse_return()
            elif token[1] == 'while':
                return self.parse_while()
            elif token[1] == 'for':
                return self.parse_for()
            elif token[1] == 'loop':
                return self.parse_loop()
        
        return self.parse_expression_statement()
    
    def parse_function(self) -> Dict:
        self.advance()  # fn
        name = self.expect('IDENT')[1]
        self.expect('LPAREN')
        
        args = []
        while self.current()[0] != 'RPAREN':
            arg_name = self.expect('IDENT')[1]
            
            # Optional type annotation
            arg_type = 'Dynamic'
            if self.current()[0] == 'COLON':
                self.advance()
                arg_type = self.expect('IDENT')[1]
            
            args.append({'name': arg_name, 'type': arg_type})
            
            if self.current()[0] == 'COMMA':
                self.advance()
            if self.current()[0] == 'COMMA':
                self.advance()
        
        self.expect('RPAREN')
        
        return_type = 'Void'
        if self.current()[0] == 'ARROW':
            self.advance()
            return_type = self.expect('IDENT')[1]
        
        body = self.parse_block()
        
        return {
            'type': 'function',
            'name': name,
            'args': args,
            'return_type': return_type,
            'body': body
        }
    
    def parse_block(self) -> List[Dict]:
        statements = []
        
        # Indented block
        if self.current()[0] == 'COLON':
            self.advance()
            while self.current()[0] not in ('EOF',):
                if self.current()[0] == 'KEYWORD' and self.current()[1] in ('else', 'end', 'fn', 'let', 'mut', 'if', 'match', 'while', 'for', 'loop', 'struct', 'enum'):
                    break
                statements.append(self.parse_statement())
        elif self.current()[0] == 'LBRACE':
            self.advance()
            while self.current()[0] != 'RBRACE' and self.current()[0] != 'EOF':
                statements.append(self.parse_statement())
            self.expect('RBRACE')
        
        return statements
    
    def parse_variable_decl(self) -> Dict:
        mutable = self.advance()[1] == 'mut'
        name = self.expect('IDENT')[1]
        
        var_type = None
        if self.current()[0] == 'COLON':
            self.advance()
            var_type = self.expect('IDENT')[1]
        
        self.expect('EQ')
        value = self.parse_expression()
        
        return {
            'type': 'variable_decl',
            'mutable': mutable,
            'name': name,
            'var_type': var_type,
            'value': value
        }
    
    def parse_if(self) -> Dict:
        self.advance()  # if
        condition = self.parse_expression()
        
        then_branch = self.parse_block()
        
        else_branch = None
        if self.current()[0] == 'KEYWORD' and self.current()[1] == 'else':
            self.advance()
            else_branch = self.parse_block()
        
        return {
            'type': 'if',
            'condition': condition,
            'then_branch': then_branch,
            'else_branch': else_branch
        }
    
    def parse_match(self) -> Dict:
        self.advance()  # match
        expr = self.parse_expression()
        
        self.expect('LBRACE')
        
        arms = []
        while self.current()[0] != 'RBRACE':
            pattern = self.parse_pattern()
            self.expect('ARROW')
            body = self.parse_block()
            arms.append({'pattern': pattern, 'body': body})
            if self.current()[0] == 'COMMA':
                self.advance()
        
        self.expect('RBRACE')
        
        return {
            'type': 'match',
            'expr': expr,
            'arms': arms
        }
    
    def parse_pattern(self) -> Dict:
        token = self.current()
        
        if token[0] == 'NUMBER':
            self.advance()
            return {'type': 'literal', 'value': token[1]}
        elif token[0] == 'STRING':
            self.advance()
            return {'type': 'literal', 'value': token[1]}
        elif token[0] == 'IDENT':
            self.advance()
            return {'type': 'identifier', 'value': token[1]}
        
        return {'type': 'wildcard'}
    
    def parse_return(self) -> Dict:
        self.advance()  # return
        value = None
        if self.current()[0] not in ('EOF', 'KEYWORD', 'RBRACE', 'COLON'):
            value = self.parse_expression()
        
        return {'type': 'return', 'value': value}
    
    def parse_while(self) -> Dict:
        self.advance()  # while
        condition = self.parse_expression()
        body = self.parse_block()
        
        return {'type': 'while', 'condition': condition, 'body': body}
    
    def parse_for(self) -> Dict:
        self.advance()  # for
        var = self.expect('IDENT')[1]
        self.expect('KEYWORD')  # in
        iter_expr = self.parse_expression()
        body = self.parse_block()
        
        return {'type': 'for', 'variable': var, 'iter': iter_expr, 'body': body}
    
    def parse_loop(self) -> Dict:
        self.advance()  # loop
        body = self.parse_block()
        
        return {'type': 'loop', 'body': body}
    
    def parse_expression_statement(self) -> Dict:
        expr = self.parse_expression()
        return {'type': 'expr', 'expr': expr}
    
    def parse_expression(self) -> Dict:
        return self.parse_or()
    
    def parse_or(self) -> Dict:
        left = self.parse_and()
        
        while self.current()[0] == 'LOGICAL' and self.current()[1] == '||':
            op = self.advance()[1]
            right = self.parse_and()
            left = {'type': 'binary', 'op': op, 'left': left, 'right': right}
        
        return left
    
    def parse_and(self) -> Dict:
        left = self.parse_equality()
        
        while self.current()[0] == 'LOGICAL' and self.current()[1] == '&&':
            op = self.advance()[1]
            right = self.parse_equality()
            left = {'type': 'binary', 'op': op, 'left': left, 'right': right}
        
        return left
    
    def parse_equality(self) -> Dict:
        left = self.parse_comparison()
        
        while self.current()[0] == 'COMPARE' and self.current()[1] in ('==', '!='):
            op = self.advance()[1]
            right = self.parse_comparison()
            left = {'type': 'binary', 'op': op, 'left': left, 'right': right}
        
        return left
    
    def parse_comparison(self) -> Dict:
        left = self.parse_addition()
        
        while self.current()[0] == 'COMPARE' and self.current()[1] in ('<', '>', '<=', '>='):
            op = self.advance()[1]
            right = self.parse_addition()
            left = {'type': 'binary', 'op': op, 'left': left, 'right': right}
        
        return left
    
    def parse_addition(self) -> Dict:
        left = self.parse_multiplication()
        
        while self.current()[0] in ('PLUS', 'MINUS'):
            op = self.advance()[1]
            right = self.parse_multiplication()
            left = {'type': 'binary', 'op': op, 'left': left, 'right': right}
        
        return left
    
    def parse_multiplication(self) -> Dict:
        left = self.parse_unary()
        
        while self.current()[0] in ('STAR', 'SLASH', 'PERCENT'):
            op = self.advance()[1]
            right = self.parse_unary()
            left = {'type': 'binary', 'op': op, 'left': left, 'right': right}
        
        return left
    
    def parse_unary(self) -> Dict:
        if self.current()[0] in ('BANG', 'MINUS'):
            op = self.advance()[1]
            operand = self.parse_unary()
            return {'type': 'unary', 'op': op, 'operand': operand}
        
        return self.parse_call()
    
    def parse_call(self) -> Dict:
        left = self.parse_primary()
        
        while self.current()[0] == 'LPAREN':
            self.advance()
            args = []
            while self.current()[0] != 'RPAREN':
                args.append(self.parse_expression())
                if self.current()[0] == 'COMMA':
                    self.advance()
            self.expect('RPAREN')
            left = {'type': 'call', 'func': left, 'args': args}
        
        return left
    
    def parse_primary(self) -> Dict:
        token = self.current()
        
        if token[0] == 'NUMBER':
            self.advance()
            return {'type': 'literal', 'value': token[1]}
        
        if token[0] == 'STRING':
            self.advance()
            return {'type': 'literal', 'value': token[1]}
        
        if token[0] == 'IDENT':
            self.advance()
            return {'type': 'identifier', 'name': token[1]}
        
        if token[0] == 'LPAREN':
            self.advance()
            expr = self.parse_expression()
            self.expect('RPAREN')
            return expr
        
        if token[0] == 'KEYWORD':
            if token[1] == 'true':
                self.advance()
                return {'type': 'literal', 'value': True}
            if token[1] == 'false':
                self.advance()
                return {'type': 'literal', 'value': False}
        
        raise SyntaxError(f"Unexpected token: {token}")


class Interpreter:
    """Simple interpreter for OmniLang"""
    
    def __init__(self):
        self.variables: Dict[str, Any] = {}
        self.functions: Dict[str, Dict] = {}
        self.builtins = {
            'print': self._builtin_print,
            'println': self._builtin_println,
            'sqrt': lambda x: x ** 0.5,
            'abs': abs,
            'len': len,
            'range': range,
            'str': str,
            'int': int,
            'float': float,
            'bool': bool,
        }
    
    def _builtin_print(self, *args):
        print(*args, end='')
    
    def _builtin_println(self, *args):
        print(*args)
    
    def interpret(self, ast: List[Dict]):
        for stmt in ast:
            self.execute(stmt)
    
    def execute(self, node: Dict) -> Any:
        node_type = node.get('type')
        
        if node_type == 'function':
            self.functions[node['name']] = node
            return None
        
        if node_type == 'variable_decl':
            value = self.evaluate(node['value'])
            self.variables[node['name']] = value
            return value
        
        if node_type == 'return':
            if node['value']:
                return self.evaluate(node['value'])
            return None
        
        if node_type == 'if':
            cond = self.evaluate(node['condition'])
            if self.is_truthy(cond):
                for stmt in node['then_branch']:
                    result = self.execute(stmt)
                    if result is not None:
                        return result
            elif node.get('else_branch'):
                for stmt in node['else_branch']:
                    result = self.execute(stmt)
                    if result is not None:
                        return result
            return None
        
        if node_type == 'while':
            while self.is_truthy(self.evaluate(node['condition'])):
                for stmt in node['body']:
                    result = self.execute(stmt)
                    if result is not None:
                        return result
            return None
        
        if node_type == 'for':
            iter_val = self.evaluate(node['iter'])
            for item in iter_val:
                self.variables[node['variable']] = item
                for stmt in node['body']:
                    result = self.execute(stmt)
                    if result is not None:
                        return result
            return None
        
        if node_type == 'loop':
            while True:
                for stmt in node['body']:
                    result = self.execute(stmt)
                    if result is not None:
                        return result
            return None
        
        if node_type == 'match':
            val = self.evaluate(node['expr'])
            for arm in node['arms']:
                if self.match_pattern(arm['pattern'], val):
                    for stmt in arm['body']:
                        result = self.execute(stmt)
                        if result is not None:
                            return result
            return None
        
        if node_type == 'expr':
            return self.evaluate(node['expr'])
        
        return None
    
    def evaluate(self, node: Dict) -> Any:
        if node is None:
            return None
        
        node_type = node.get('type')
        
        if node_type == 'literal':
            return node['value']
        
        if node_type == 'identifier':
            name = node['name']
            if name in self.variables:
                return self.variables[name]
            if name in self.functions:
                return self.functions[name]
            if name in self.builtins:
                return self.builtins[name]
            raise NameError(f"Unknown identifier: {name}")
        
        if node_type == 'binary':
            left = self.evaluate(node['left'])
            right = self.evaluate(node['right'])
            op = node['op']
            
            ops = {
                '+': lambda a, b: a + b,
                '-': lambda a, b: a - b,
                '*': lambda a, b: a * b,
                '/': lambda a, b: a / b,
                '%': lambda a, b: a % b,
                '==': lambda a, b: a == b,
                '!=': lambda a, b: a != b,
                '<': lambda a, b: a < b,
                '>': lambda a, b: a > b,
                '<=': lambda a, b: a <= b,
                '>=': lambda a, b: a >= b,
                '&&': lambda a, b: a and b,
                '||': lambda a, b: a or b,
            }
            
            if op in ops:
                return ops[op](left, right)
            raise ValueError(f"Unknown operator: {op}")
        
        if node_type == 'unary':
            operand = self.evaluate(node['operand'])
            op = node['op']
            
            if op == '-':
                return -operand
            if op == '!':
                return not operand
            
            raise ValueError(f"Unknown unary operator: {op}")
        
        if node_type == 'call':
            func = self.evaluate(node['func'])
            args = [self.evaluate(arg) for arg in node['args']]
            
            if callable(func):
                return func(*args)
            
            # User-defined function
            if isinstance(func, dict):
                old_vars = self.variables.copy()
                for param, arg in zip(func['args'], args):
                    self.variables[param['name']] = arg
                
                result = None
                for stmt in func['body']:
                    result = self.execute(stmt)
                
                self.variables = old_vars
                return result
            
            raise TypeError(f"Cannot call {type(func)}")
        
        return None
    
    def match_pattern(self, pattern: Dict, value: Any) -> bool:
        pattern_type = pattern.get('type')
        
        if pattern_type == 'wildcard':
            return True
        
        if pattern_type == 'literal':
            return pattern['value'] == value
        
        if pattern_type == 'identifier':
            return True
        
        return False
    
    def is_truthy(self, value: Any) -> bool:
        if value is None:
            return False
        if isinstance(value, bool):
            return value
        if isinstance(value, (int, float)):
            return value != 0
        if isinstance(value, (str, list, dict)):
            return len(value) != 0
        return True


def main():
    if len(sys.argv) < 2:
        print("Usage: python omnilang.py <file.omni>")
        sys.exit(1)
    
    filename = sys.argv[1]
    
    try:
        with open(filename, 'r') as f:
            source = f.read()
    except FileNotFoundError:
        print(f"Error: File not found: {filename}")
        sys.exit(1)
    except IOError as e:
        print(f"Error reading file: {e}")
        sys.exit(1)
    
    print(f"OmniLang v0.1 - Running {filename}")
    print("=" * 50)
    
    # Lexical analysis
    print("Lexing...")
    lexer = Lexer(source)
    tokens = lexer.tokenize()
    print(f"  Generated {len(tokens)} tokens")
    
    # Parsing
    print("Parsing...")
    parser = Parser(tokens)
    try:
        ast = parser.parse()
        print(f"  Generated {len(ast)} statements")
    except SyntaxError as e:
        print(f"Syntax Error: {e}")
        sys.exit(1)
    
    # Interpretation
    print("Executing...")
    print("-" * 50)
    
    interpreter = Interpreter()
    try:
        interpreter.interpret(ast)
        
        # Call main function if it exists
        if 'main' in interpreter.functions:
            print("Calling main()...")
            main_func = interpreter.functions['main']
            for stmt in main_func['body']:
                result = interpreter.execute(stmt)
    except Exception as e:
        print(f"Runtime Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    
    print("-" * 50)
    print("Program completed successfully!")


if __name__ == '__main__':
    main()
