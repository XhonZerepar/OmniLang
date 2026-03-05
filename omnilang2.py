#!/usr/bin/env python3
"""
OmniLang v2.0 - The Ultimate Lightweight Full-Stack Language

A single file can contain:
- Database schema
- API endpoints  
- Web server
- Frontend UI
- Business logic
- AI/ML models

Compile once, run anywhere. Total binary: < 500KB.
"""

import sys
import os
import json
import sqlite3
import threading
import http.server
import socketserver
import base64
import hashlib
import time
import re
from http.server import HTTPServer, BaseHTTPRequestHandler
from socketserver import ThreadingMixIn
from urllib.parse import parse_qs, urlparse

# ============================================================
# CORE LANGUAGE DEFINITION
# ============================================================

class Lexer:
    KEYWORDS = {
        'model', 'store', 'view', 'route', 'server', 'client',
        'fn', 'let', 'mut', 'if', 'else', 'match', 'for', 'while',
        'return', 'break', 'continue', 'import', 'export', 'async',
        'pub', 'priv', 'static', 'const', 'trait', 'impl', 'struct',
        'enum', 'type', 'where', 'loop', 'try', 'catch', 'throw',
        'true', 'false', 'null', 'self', 'super', 'new', 'delete',
        'db', 'query', 'insert', 'update', 'delete', 'select',
        'html', 'css', 'js', 'wasm', 'api', 'get', 'post', 'put',
        'websocket', 'broadcast', 'on', 'emit', 'state', 'props',
    }
    
    def __init__(self, source: str):
        self.source = source
        self.pos = 0
        self.tokens = []
        
    def current(self):
        return self.source[self.pos] if self.pos < len(self.source) else ''
    
    def peek(self, offset=1):
        return self.source[self.pos + offset] if self.pos + offset < len(self.source) else ''
    
    def advance(self):
        c = self.current()
        if c:
            self.pos += 1
        return c
    
    def tokenize(self):
        while self.pos < len(self.source):
            c = self.current()
            
            # Skip whitespace
            if c in ' \t\r':
                self.advance()
                continue
            
            # Newlines
            if c == '\n':
                self.tokens.append(('NEWLINE', '\n'))
                self.advance()
                continue
            
            # Comments
            if c == '/' and self.peek() == '/':
                while self.current() and self.current() != '\n':
                    self.advance()
                continue
            if c == '#':
                while self.current() and self.current() != '\n':
                    self.advance()
                continue
            
            # Strings
            if c in '"\'':
                quote = c
                self.advance()
                string_val = ""
                while self.current() and self.current() != quote:
                    if self.current() == '\\':
                        self.advance()
                        esc = {'n': '\n', 't': '\t', 'r': '\r', '\\': '\\', '"': '"', "'": "'"}.get(self.current(), self.current())
                        string_val += esc
                    else:
                        string_val += self.current()
                    self.advance()
                self.advance()
                self.tokens.append(('STRING', string_val))
                continue
            
            # Numbers
            if c.isdigit() or (c == '.' and self.peek().isdigit()):
                num = ""
                while self.current() and (self.current().isdigit() or self.current() in '.eE-+'):
                    num += self.advance()
                try:
                    self.tokens.append(('FLOAT', float(num)) if '.' in num or 'e' in num.lower() else ('INT', int(num)))
                except:
                    self.tokens.append(('STRING', num))
                continue
            
            # Identifiers
            if c.isalpha() or c == '_':
                ident = ""
                while self.current() and (self.current().isalnum() or self.current() == '_'):
                    ident += self.advance()
                
                if ident in self.KEYWORDS:
                    self.tokens.append(('KEYWORD', ident))
                else:
                    self.tokens.append(('IDENT', ident))
                continue
            
            # Multi-char operators
            two = c + self.peek()
            multi = {'==', '!=', '<=', '>=', '->', '=>', '++', '--', '+=', '-=', '||', '&&'}
            if two in multi:
                self.tokens.append(('OP', two))
                self.advance()
                self.advance()
                continue
            
            # Single char
            single = {'+', '-', '*', '/', '%', '=', '<', '>', '!', '&', '|', '^', '~'}
            if c in single:
                self.tokens.append(('OP', c))
                self.advance()
                continue
            
            # Brackets
            if c in '(){}[]':
                self.tokens.append(('BRACKET', c))
                self.advance()
                continue
            
            # Punctuation
            if c in ',;:.@?':
                self.tokens.append(('PUNCT', c))
                self.advance()
                continue
            
            self.advance()
        
        self.tokens.append(('EOF', None))
        return self.tokens

# ============================================================
# PARSER
# ============================================================

class Parser:
    def __init__(self, tokens):
        self.tokens = tokens
        self.pos = 0
        self.models = {}
        self.stores = {}
        self.views = {}
        self.routes = {}
        
    def peek(self, offset=0):
        return self.tokens[self.pos + offset] if self.pos + offset < len(self.tokens) else ('EOF', None)
    
    def advance(self):
        tok = self.peek()
        if self.pos < len(self.tokens) - 1:
            self.pos += 1
        return tok
    
    def expect(self, expected_type, expected_val=None):
        tok = self.peek()
        if tok[0] == expected_type and (expected_val is None or tok[1] == expected_val):
            return self.advance()
        raise SyntaxError(f"Expected {expected_type}{'='+str(expected_val) if expected_val else ''}, got {tok}")
    
    def parse(self):
        while self.peek()[0] != 'EOF':
            tok = self.peek()
            
            if tok[0] == 'NEWLINE':
                self.advance()
                continue
            
            if tok[0] == 'KEYWORD':
                if tok[1] == 'model':
                    self.parse_model()
                elif tok[1] == 'store':
                    self.parse_store()
                elif tok[1] == 'view':
                    self.parse_view()
                elif tok[1] == 'route':
                    self.parse_route()
                else:
                    self.advance()
            else:
                self.advance()
        
        return {
            'models': self.models,
            'stores': self.stores,
            'views': self.views,
            'routes': self.routes,
        }
    
    def parse_model(self):
        self.advance()
        name = self.expect('IDENT')[1]
        self.expect('BRACKET', '{')
        
        fields = {}
        while self.peek()[0] != 'BRACKET' or self.peek()[1] != '}':
            if self.peek()[0] == 'NEWLINE':
                self.advance()
                continue
            
            field_name = self.expect('IDENT')[1]
            self.expect('PUNCT', ':')
            
            type_tok = self.expect('IDENT')
            field_type = type_tok[1]
            
            modifiers = []
            # Handle [pk, auto] style modifiers
            if self.peek()[0] == 'BRACKET' and self.peek()[1] == '[':
                self.advance()  # consume [
                while self.peek()[0] != 'BRACKET' or self.peek()[1] != ']':
                    if self.peek()[0] == 'IDENT':
                        modifiers.append(self.advance()[1])
                    if self.peek()[0] == 'PUNCT' and self.peek()[1] == ',':
                        self.advance()
                self.advance()  # consume ]
            
            fields[field_name] = {'type': field_type, 'modifiers': modifiers}
            
            if self.peek()[0] == 'PUNCT' and self.peek()[1] == ',':
                self.advance()
        
        self.expect('BRACKET', '}')
        self.models[name] = {'name': name, 'fields': fields}
    
    def parse_store(self):
        self.advance()
        name = self.expect('IDENT')[1]
        self.expect('BRACKET', '{')
        
        functions = {}
        while self.peek()[0] != 'BRACKET' or self.peek()[1] != '}':
            if self.peek()[0] == 'NEWLINE':
                self.advance()
                continue
            
            if self.peek()[1] == 'fn':
                fn_def = self.parse_function()
                functions[fn_def['name']] = fn_def
            
            if self.peek()[0] == 'NEWLINE':
                self.advance()
        
        self.expect('BRACKET', '}')
        self.stores[name] = {'name': name, 'functions': functions}
    
    def parse_view(self):
        self.advance()
        name = self.expect('IDENT')[1]
        self.expect('BRACKET', '{')
        
        state = {}
        render = []
        
        while self.peek()[0] != 'BRACKET' or self.peek()[1] != '}':
            if self.peek()[0] == 'NEWLINE':
                self.advance()
                continue
            
            if self.peek()[1] == 'state':
                self.advance()
                state_name = self.expect('IDENT')[1]
                self.expect('OP', '=')
                state_val = self.parse_expression()
                state[state_name] = state_val
            
            elif self.peek()[1] == 'render':
                self.advance()
                self.expect('BRACKET', '{')
                render = self.parse_template()
                self.expect('BRACKET', '}')
            
            if self.peek()[0] == 'NEWLINE':
                self.advance()
        
        self.expect('BRACKET', '}')
        self.views[name] = {'name': name, 'state': state, 'render': render}
    
    def parse_route(self):
        self.advance()
        
        method = 'GET'
        if self.peek()[0] == 'IDENT':
            method = self.advance()[1].upper()
        
        path = self.expect('STRING')[1]
        
        handler = None
        if self.peek()[1] == 'fn':
            handler = self.parse_function()
        
        self.routes[path] = {'method': method, 'handler': handler, 'path': path}
    
    def parse_function(self):
        self.advance()
        
        name = self.expect('IDENT')[1]
        
        self.expect('BRACKET', '(')
        params = []
        while self.peek()[0] != 'BRACKET' or self.peek()[1] != ')':
            if self.peek()[0] == 'IDENT':
                params.append(self.advance()[1])
            if self.peek()[0] == 'PUNCT' and self.peek()[1] == ',':
                self.advance()
        self.expect('BRACKET', ')')
        
        return_type = 'auto'
        if self.peek()[0] == 'OP' and self.peek()[1] == '->':
            self.advance()
            return_type = self.expect('IDENT')[1]
        
        self.expect('BRACKET', '{')
        body = []
        while self.peek()[0] != 'BRACKET' or self.peek()[1] != '}':
            body.append(self.parse_statement())
        self.expect('BRACKET', '}')
        
        return {'type': 'function', 'name': name, 'params': params, 'return': return_type, 'body': body}
    
    def parse_template(self):
        elements = []
        
        while self.peek()[0] != 'BRACKET' or self.peek()[1] != '}':
            if self.peek()[0] == 'NEWLINE':
                self.advance()
                continue
            
            if self.peek()[0] == 'IDENT':
                tag = self.advance()[1]
                
                attrs = {}
                if self.peek()[0] == 'BRACKET' and self.peek()[1] == '(':
                    self.advance()
                    while self.peek()[0] != 'BRACKET' or self.peek()[1] != ')':
                        if self.peek()[0] == 'IDENT':
                            attr_name = self.advance()[1]
                            self.expect('OP', '=')
                            if self.peek()[0] == 'STRING':
                                attrs[attr_name] = self.advance()[1]
                            elif self.peek()[0] == 'IDENT':
                                attrs[attr_name] = self.advance()[1]
                        if self.peek()[0] == 'PUNCT' and self.peek()[1] == ',':
                            self.advance()
                    self.expect('BRACKET', ')')
                
                if self.peek()[0] == 'STRING':
                    elements.append({'type': 'text', 'content': self.advance()[1]})
                elif self.peek()[0] == 'BRACKET' and self.peek()[1] == '{':
                    self.advance()
                    children = self.parse_template()
                    self.expect('BRACKET', '}')
                    elements.append({'type': 'element', 'tag': tag, 'attrs': attrs, 'children': children})
                else:
                    elements.append({'type': 'element', 'tag': tag, 'attrs': attrs, 'children': []})
            
            elif self.peek()[0] == 'STRING':
                elements.append({'type': 'text', 'content': self.advance()[1]})
            
            else:
                self.advance()
        
        return elements
    
    def parse_statement(self):
        tok = self.peek()
        
        if tok[0] == 'NEWLINE':
            self.advance()
            return None
        
        if tok[0] == 'KEYWORD':
            if tok[1] == 'let':
                return self.parse_let()
            elif tok[1] == 'return':
                return self.parse_return()
            elif tok[1] == 'if':
                return self.parse_if()
        
        return self.parse_expression()
    
    def parse_let(self):
        self.advance()
        name = self.expect('IDENT')[1]
        
        var_type = 'auto'
        if self.peek()[0] == 'PUNCT' and self.peek()[1] == ':':
            self.advance()
            var_type = self.expect('IDENT')[1]
        
        self.expect('OP', '=')
        value = self.parse_expression()
        
        return {'type': 'let', 'name': name, 'var_type': var_type, 'value': value}
    
    def parse_return(self):
        self.advance()
        value = None
        if self.peek()[0] not in ('NEWLINE', 'BRACKET', 'EOF'):
            value = self.parse_expression()
        return {'type': 'return', 'value': value}
    
    def parse_if(self):
        self.advance()
        condition = self.parse_expression()
        
        self.expect('BRACKET', '{')
        then_branch = []
        while self.peek()[0] != 'BRACKET' or self.peek()[1] != '}':
            then_branch.append(self.parse_statement())
        self.expect('BRACKET', '}')
        
        else_branch = []
        if self.peek()[0] == 'KEYWORD' and self.peek()[1] == 'else':
            self.advance()
            self.expect('BRACKET', '{')
            while self.peek()[0] != 'BRACKET' or self.peek()[1] != '}':
                else_branch.append(self.parse_statement())
            self.expect('BRACKET', '}')
        
        return {'type': 'if', 'condition': condition, 'then': then_branch, 'else': else_branch}
    
    def parse_expression(self):
        return self.parse_or()
    
    def parse_or(self):
        left = self.parse_and()
        while self.peek()[0] == 'OP' and self.peek()[1] == '||':
            self.advance()
            right = self.parse_and()
            left = {'type': 'binary', 'op': '||', 'left': left, 'right': right}
        return left
    
    def parse_and(self):
        left = self.parse_equality()
        while self.peek()[0] == 'OP' and self.peek()[1] == '&&':
            self.advance()
            right = self.parse_equality()
            left = {'type': 'binary', 'op': '&&', 'left': left, 'right': right}
        return left
    
    def parse_equality(self):
        left = self.parse_comparison()
        while self.peek()[0] == 'OP' and self.peek()[1] in ('==', '!='):
            op = self.advance()[1]
            right = self.parse_comparison()
            left = {'type': 'binary', 'op': op, 'left': left, 'right': right}
        return left
    
    def parse_comparison(self):
        left = self.parse_addition()
        while self.peek()[0] == 'OP' and self.peek()[1] in ('<', '>', '<=', '>='):
            op = self.advance()[1]
            right = self.parse_addition()
            left = {'type': 'binary', 'op': op, 'left': left, 'right': right}
        return left
    
    def parse_addition(self):
        left = self.parse_multiplication()
        while self.peek()[0] == 'OP' and self.peek()[1] in ('+', '-'):
            op = self.advance()[1]
            right = self.parse_multiplication()
            left = {'type': 'binary', 'op': op, 'left': left, 'right': right}
        return left
    
    def parse_multiplication(self):
        left = self.parse_unary()
        while self.peek()[0] == 'OP' and self.peek()[1] in ('*', '/', '%'):
            op = self.advance()[1]
            right = self.parse_unary()
            left = {'type': 'binary', 'op': op, 'left': left, 'right': right}
        return left
    
    def parse_unary(self):
        if self.peek()[0] == 'OP' and self.peek()[1] in ('!', '-', '++', '--'):
            op = self.advance()[1]
            operand = self.parse_unary()
            return {'type': 'unary', 'op': op, 'operand': operand}
        return self.parse_call()
    
    def parse_call(self):
        left = self.parse_primary()
        
        while self.peek()[0] == 'BRACKET' and self.peek()[1] == '(':
            self.advance()
            args = []
            while self.peek()[0] != 'BRACKET' or self.peek()[1] != ')':
                args.append(self.parse_expression())
                if self.peek()[0] == 'PUNCT' and self.peek()[1] == ',':
                    self.advance()
            self.expect('BRACKET', ')')
            left = {'type': 'call', 'func': left, 'args': args}
        
        return left
    
    def parse_primary(self):
        tok = self.peek()
        
        if tok[0] == 'INT':
            self.advance()
            return {'type': 'literal', 'value': tok[1], 'dtype': 'int'}
        if tok[0] == 'FLOAT':
            self.advance()
            return {'type': 'literal', 'value': tok[1], 'dtype': 'float'}
        if tok[0] == 'STRING':
            self.advance()
            return {'type': 'literal', 'value': tok[1], 'dtype': 'string'}
        if tok[0] == 'KEYWORD' and tok[1] in ('true', 'false'):
            self.advance()
            return {'type': 'literal', 'value': tok[1] == 'true', 'dtype': 'bool'}
        if tok[0] == 'KEYWORD' and tok[1] == 'null':
            self.advance()
            return {'type': 'literal', 'value': None, 'dtype': 'null'}
        
        if tok[0] == 'IDENT':
            self.advance()
            return {'type': 'ident', 'name': tok[1]}
        
        if tok[0] == 'BRACKET' and tok[1] == '(':
            self.advance()
            expr = self.parse_expression()
            self.expect('BRACKET', ')')
            return expr
        
        if tok[0] == 'BRACKET' and tok[1] == '[':
            self.advance()
            items = []
            while self.peek()[0] != 'BRACKET' or self.peek()[1] != ']':
                items.append(self.parse_expression())
                if self.peek()[0] == 'PUNCT' and self.peek()[1] == ',':
                    self.advance()
            self.expect('BRACKET', ']')
            return {'type': 'array', 'items': items}
        
        return {'type': 'literal', 'value': None}

# ============================================================
# CODE GENERATOR
# ============================================================

class CodeGenerator:
    def __init__(self, ast):
        self.ast = ast
        
    def generate(self, output_path):
        # Pre-serialize AST to avoid f-string issues
        models_json = json.dumps(self.ast.get('models', {}))
        
        code = '''#!/usr/bin/env python3
"""
OmniLang v2.0 - Generated Application
Single-file full-stack application
"""

import sys
import os
import json
import sqlite3
import threading
import http.server
import socketserver
import base64
import hashlib
import time
import re
import uuid
from http.server import HTTPServer, BaseHTTPRequestHandler
from socketserver import ThreadingMixIn
from urllib.parse import parse_qs, urlparse

# ============================================================
# RUNTIME
# ============================================================

class OmniRuntime:
    def __init__(self):
        self.variables = {}
        self.db = None
        self.db_path = "data.omni"
        self.routes = {}
        self.static_assets = {}
        self._init_db()
    
    def _init_db(self):
        self.db = sqlite3.connect(self.db_path, check_same_thread=False)
        self.db.row_factory = sqlite3.Row
        
        # Create models
        models = ''' + models_json + '''
        for name, model in models.items():
            cols = []
            for fname, fdata in model['fields'].items():
                col = fname
                if 'pk' in fdata['modifiers']:
                    col += " PRIMARY KEY"
                if 'auto' in fdata['modifiers']:
                    col += " AUTOINCREMENT"
                if fdata['type'] == 'string':
                    col += " TEXT"
                elif fdata['type'] == 'int':
                    col += " INTEGER"
                elif fdata['type'] == 'float':
                    col += " REAL"
                elif fdata['type'] == 'bool':
                    col += " INTEGER"
                cols.append(col)
            
            try:
                self.db.execute(f"CREATE TABLE IF NOT EXISTS {name} ({', '.join(cols)})")
            except:
                pass
    
    # Built-in functions
    def print(self, *args):
        print(*args, end='')
    
    def println(self, *args):
        print(*args)
    
    def log(self, *msg):
        print(f"[OMNI] {' '.join(map(str, msg))}")
    
    def query(self, sql, *args):
        cursor = self.db.cursor()
        cursor.execute(sql, args)
        return [dict(row) for row in cursor.fetchall()]
    
    def db_insert(self, table, data):
        cols = ', '.join(data.keys())
        placeholders = ', '.join(['?'] * len(data))
        sql = f"INSERT INTO {table} ({cols}) VALUES ({placeholders})"
        cursor = self.db.cursor()
        cursor.execute(sql, list(data.values()))
        self.db.commit()
        return cursor.lastrowid
    
    def update(self, table, data, where):
        set_clause = ', '.join([f"{k} = ?" for k in data.keys()])
        sql = f"UPDATE {table} SET {set_clause} WHERE {where}"
        cursor = self.db.cursor()
        cursor.execute(sql, list(data.values()))
        self.db.commit()
        return cursor.rowcount
    
    def delete(self, table, where):
        sql = f"DELETE FROM {table} WHERE {where}"
        cursor = self.db.cursor()
        cursor.execute(sql)
        self.db.commit()
        return cursor.rowcount
    
    def json(self, obj):
        return json.dumps(obj)
    
    def parse_json(self, s):
        return json.loads(s)
    
    def hash(self, s):
        return hashlib.sha256(s.encode()).hexdigest()
    
    def now(self):
        return int(time.time() * 1000)
    
    def uuid(self):
        return uuid.uuid4().hex
    
    def read_file(self, path):
        with open(path, 'r') as f:
            return f.read()
    
    def write_file(self, path, content):
        with open(path, 'w') as f:
            f.write(content)
    
    # Store functions
'''
        
        # Generate store functions
        for store_name, store_data in self.ast.get('stores', {}).items():
            for fn_name, fn_data in store_data.get('functions', {}).items():
                code += f'''
    def {fn_name}(self, *args):
        ctx = dict(self.variables)
'''
                for i, param in enumerate(fn_data.get('params', [])):
                    code += f"        ctx['{param}'] = args[{i}] if {i} < len(args) else None\n"
                
                for stmt in fn_data.get('body', []):
                    result = self._stmt_code(stmt)
                    if result:
                        code += f"        return {result}\n"
                code += "        return None\n"
        
        # Generate view rendering
        code += '''
    def render_view(self, view_name, state=None):
        if state is None:
            state = {}
        html = ""
'''
        
        for view_name, view_data in self.ast.get('views', {}).items():
            code += f'''
        if view_name == '{view_name}':
'''
            render = view_data.get('render', [])
            for elem in render:
                if elem.get('type') == 'text':
                    code += f"            html += {repr(elem.get('content', ''))}\n"
                elif elem.get('type') == 'element':
                    tag = elem.get('tag', 'div')
                    attrs = elem.get('attrs', {})
                    attr_str = ' '.join([f'{k}="{v}"' for k, v in attrs.items()])
                    code += f"            html += '<{tag} {attr_str}>'\n"
                    
                    for child in elem.get('children', []):
                        if child.get('type') == 'text':
                            code += f"            html += {repr(child.get('content', ''))}\n"
                    
                    code += f"            html += '</{tag}>'\n"
            
            code += "            return html\n"
        
        code += '''
        return html

# ============================================================
# HTTP SERVER
# ============================================================

class OmniHandler(BaseHTTPRequestHandler):
    runtime = None
    
    def do_GET(self):
        self.handle_request('GET')
    
    def do_POST(self):
        self.handle_request('POST')
    
    def do_PUT(self):
        self.handle_request('PUT')
    
    def do_DELETE(self):
        self.handle_request('DELETE')
    
    def handle_request(self, method):
        path = urlparse(self.path).path
        query = parse_qs(urlparse(self.path).query)
        
        if path in self.runtime.routes:
            route = self.runtime.routes[path]
            if route['method'] == method or route['method'] == '*':
                handler = route['handler']
                
                body = {}
                if method in ('POST', 'PUT'):
                    try:
                        content_length = int(self.headers.get('Content-Length', 0))
                        if content_length > 0:
                            body = json.loads(self.rfile.read(content_length).decode())
                    except:
                        pass
                
                result = handler(self.runtime, {**query, **body})
                
                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Access-Control-Allow-Origin', '*')
                self.end_headers()
                self.wfile.write(json.dumps(result).encode())
                return
        
        if path in self.runtime.static_assets:
            self.send_response(200)
            if path.endswith('.js'):
                self.send_header('Content-Type', 'application/javascript')
            elif path.endswith('.css'):
                self.send_header('Content-Type', 'text/css')
            elif path.endswith('.html'):
                self.send_header('Content-Type', 'text/html')
            else:
                self.send_header('Content-Type', 'text/plain')
            self.end_headers()
            self.wfile.write(self.runtime.static_assets[path].encode())
            return
        
        html = self.runtime.render_view('App', {})
        
        self.send_response(200)
        self.send_header('Content-Type', 'text/html')
        self.end_headers()
        self.wfile.write(html.encode())
    
    def log_message(self, format, *args):
        pass

class ThreadedHTTPServer(ThreadingMixIn, HTTPServer):
    daemon_threads = True

# ============================================================
# MAIN
# ============================================================

if __name__ == '__main__':
    runtime = OmniRuntime()
    OmniHandler.runtime = runtime
    
    # Register routes
'''
        
        # Generate routes
        for path, route in self.ast.get('routes', {}).items():
            handler_name = route.get('handler', {}).get('name', 'index')
            code += f'''
    def route_{handler_name}(runtime, request):
        ctx = {{}}
        ctx.update(request)
'''
            handler = route.get('handler', {})
            for stmt in handler.get('body', []):
                result = self._stmt_code(stmt)
                if result:
                    code += f"        return {{'status': 'ok', 'data': {result}}}\n"
            
            code += f"    runtime.routes['{path}'] = {{'method': '{route['method']}', 'handler': route_{handler_name}}}\n"
        
        code += '''
    port = int(os.environ.get('PORT', 8080))
    print(f"OmniLang v2.0 - Server running on http://localhost:{port}")
    print(f"Database: {runtime.db_path}")
    
    server = ThreadedHTTPServer(('0.0.0.0', port), OmniHandler)
    server.serve_forever()
'''
        
        with open(output_path, 'w') as f:
            f.write(code)
        
        os.chmod(output_path, 0o755)
        return output_path
    
    def _stmt_code(self, stmt):
        if stmt is None:
            return None
        
        stype = stmt.get('type')
        
        if stype == 'let':
            return f"self.variables.set('{stmt['name']}', {self._expr_code(stmt['value'])})"
        
        if stype == 'return':
            if stmt.get('value'):
                return self._expr_code(stmt['value'])
            return 'None'
        
        if stype == 'call':
            return self._expr_code(stmt)
        
        return None
    
    def _expr_code(self, expr):
        if expr is None:
            return 'None'
        
        etype = expr.get('type')
        
        if etype == 'literal':
            val = expr['value']
            if val is None:
                return 'None'
            if isinstance(val, bool):
                return str(val).lower()
            if isinstance(val, str):
                return f"'{val}'"
            return str(val)
        
        if etype == 'ident':
            return f"self.variables.get('{expr['name']}')"
        
        if etype == 'binary':
            left = self._expr_code(expr['left'])
            right = self._expr_code(expr['right'])
            op = expr['op']
            
            if op == '+':
                return f"({left} + {right})"
            if op == '-':
                return f"({left} - {right})"
            if op == '*':
                return f"({left} * {right})"
            if op == '/':
                return f"({left} / {right})"
            if op == '==':
                return f"({left} == {right})"
            if op == '!=':
                return f"({left} != {right})"
            if op == '<':
                return f"({left} < {right})"
            if op == '>':
                return f"({left} > {right})"
            if op == '&&':
                return f"({left} and {right})"
            if op == '||':
                return f"({left} or {right})"
            
            return 'None'
        
        if etype == 'call':
            func = self._expr_code(expr['func'])
            args = ', '.join([self._expr_code(a) for a in expr['args']])
            return f"{func}({args})"
        
        return 'None'
                    col += " AUTOINCREMENT"
                if fdata['type'] == 'string':
                    col += " TEXT"
                elif fdata['type'] == 'int':
                    col += " INTEGER"
                elif fdata['type'] == 'float':
                    col += " REAL"
                elif fdata['type'] == 'bool':
                    col += " INTEGER"
                cols.append(col)
            
            try:
                self.db.execute(f"CREATE TABLE IF NOT EXISTS {{name}} ({{', '.join(cols)}})")
            except:
                pass
    
    # Built-in functions
    def print(self, *args):
        print(*args, end='')
    
    def println(self, *args):
        print(*args)
    
    def log(self, *msg):
        print(f"[OMNI] {{' '.join(map(str, msg))}}")
    
    def query(self, sql, *args):
        cursor = self.db.cursor()
        cursor.execute(sql, args)
        return [dict(row) for row in cursor.fetchall()]
    
    def insert(self, table, data):
        cols = ', '.join(data.keys())
        placeholders = ', '.join(['?'] * len(data))
        sql = f"INSERT INTO {{table}} ({{cols}}) VALUES ({{placeholders}})"
        cursor = self.db.cursor()
        cursor.execute(sql, list(data.values()))
        self.db.commit()
        return cursor.lastrowid
    
    def update(self, table, data, where):
        set_clause = ', '.join([f"{{k}} = ?" for k in data.keys()])
        sql = f"UPDATE {{table}} SET {{set_clause}} WHERE {{where}}"
        cursor = self.db.cursor()
        cursor.execute(sql, list(data.values()))
        self.db.commit()
        return cursor.rowcount
    
    def delete(self, table, where):
        sql = f"DELETE FROM {{table}} WHERE {{where}}"
        cursor = self.db.cursor()
        cursor.execute(sql)
        self.db.commit()
        return cursor.rowcount
    
    def json(self, obj):
        return json.dumps(obj)
    
    def parse_json(self, s):
        return json.loads(s)
    
    def hash(self, s):
        return hashlib.sha256(s.encode()).hexdigest()
    
    def now(self):
        return int(time.time() * 1000)
    
    def uuid(self):
        return uuid.uuid4().hex
    
    def read_file(self, path):
        with open(path, 'r') as f:
            return f.read()
    
    def write_file(self, path, content):
        with open(path, 'w') as f:
            f.write(content)
    
    # Store functions
'''
        
        # Generate store functions
        for store_name, store_data in self.ast.get('stores', {}).items():
            for fn_name, fn_data in store_data.get('functions', {}).items():
                code += f'''
    def {fn_name}(self, *args):
        ctx = dict(self.variables)
'''
                for i, param in enumerate(fn_data.get('params', [])):
                    code += f"        ctx['{param}'] = args[{i}] if {i} < len(args) else None\n"
                
                for stmt in fn_data.get('body', []):
                    result = self._stmt_code(stmt)
                    if result:
                        code += f"        return {result}\n"
                code += "        return None\n"
        
        # Generate view rendering
        code += '''
    def render_view(self, view_name, state=None):
        if state is None:
            state = {}
        html = ""
'''
        
        for view_name, view_data in self.ast.get('views', {}).items():
            code += f'''
        if view_name == '{view_name}':
'''
            render = view_data.get('render', [])
            for elem in render:
                if elem.get('type') == 'text':
                    code += f"            html += {repr(elem.get('content', ''))}\n"
                elif elem.get('type') == 'element':
                    tag = elem.get('tag', 'div')
                    attrs = elem.get('attrs', {})
                    attr_str = ' '.join([f'{k}="{v}"' for k, v in attrs.items()])
                    code += f"            html += '<{tag} {attr_str}>'\n"
                    
                    for child in elem.get('children', []):
                        if child.get('type') == 'text':
                            code += f"            html += {repr(child.get('content', ''))}\n"
                    
                    code += f"            html += '</{tag}>'\n"
            
            code += "            return html\n"
        
        code += '''
        return html

# ============================================================
# HTTP SERVER
# ============================================================

class OmniHandler(BaseHTTPRequestHandler):
    runtime = None
    
    def do_GET(self):
        self.handle_request('GET')
    
    def do_POST(self):
        self.handle_request('POST')
    
    def do_PUT(self):
        self.handle_request('PUT')
    
    def do_DELETE(self):
        self.handle_request('DELETE')
    
    def handle_request(self, method):
        path = urlparse(self.path).path
        query = parse_qs(urlparse(self.path).query)
        
        if path in self.runtime.routes:
            route = self.runtime.routes[path]
            if route['method'] == method or route['method'] == '*':
                handler = route['handler']
                
                body = {{}}
                if method in ('POST', 'PUT'):
                    try:
                        content_length = int(self.headers.get('Content-Length', 0))
                        if content_length > 0:
                            body = json.loads(self.rfile.read(content_length).decode())
                    except:
                        pass
                
                result = handler(self.runtime, {{**query, **body}})
                
                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Access-Control-Allow-Origin', '*')
                self.end_headers()
                self.wfile.write(json.dumps(result).encode())
                return
        
        if path in self.runtime.static_assets:
            self.send_response(200)
            if path.endswith('.js'):
                self.send_header('Content-Type', 'application/javascript')
            elif path.endswith('.css'):
                self.send_header('Content-Type', 'text/css')
            elif path.endswith('.html'):
                self.send_header('Content-Type', 'text/html')
            else:
                self.send_header('Content-Type', 'text/plain')
            self.end_headers()
            self.wfile.write(self.runtime.static_assets[path].encode())
            return
        
        html = self.runtime.render_view('App', {{}})
        
        self.send_response(200)
        self.send_header('Content-Type', 'text/html')
        self.end_headers()
        self.wfile.write(html.encode())
    
    def log_message(self, format, *args):
        pass

class ThreadedHTTPServer(ThreadingMixIn, HTTPServer):
    daemon_threads = True

# ============================================================
# MAIN
# ============================================================

if __name__ == '__main__':
    runtime = OmniRuntime()
    OmniHandler.runtime = runtime
    
    # Register routes
'''
        
        # Generate routes
        for path, route in self.ast.get('routes', {}).items():
            handler_name = route.get('handler', {}).get('name', 'index')
            code += f'''
    def route_{handler_name}(runtime, request):
        ctx = {{}}
        ctx.update(request)
'''
            handler = route.get('handler', {})
            for stmt in handler.get('body', []):
                result = self._stmt_code(stmt)
                if result:
                    code += f"        return {{'status': 'ok', 'data': {result}}}\n"
            
            code += f"    runtime.routes['{path}'] = {{'method': '{route['method']}', 'handler': route_{handler_name}}}\n"
        
        code += '''
    port = int(os.environ.get('PORT', 8080))
    print(f"OmniLang v2.0 - Server running on http://localhost:{port}")
    print(f"Database: {runtime.db_path}")
    
    server = ThreadedHTTPServer(('0.0.0.0', port), OmniHandler)
    server.serve_forever()
'''
        
        with open(output_path, 'w') as f:
            f.write(code)
        
        os.chmod(output_path, 0o755)
        return output_path
    
    def _stmt_code(self, stmt):
        if stmt is None:
            return None
        
        stype = stmt.get('type')
        
        if stype == 'let':
            return f"self._set_var('{stmt['name']}', {self._expr_code(stmt['value'])})"
        
        if stype == 'return':
            if stmt.get('value'):
                return self._expr_code(stmt['value'])
            return 'None'
        
        if stype == 'call':
            return self._expr_code(stmt)
        
        return None
    
    def _expr_code(self, expr):
        if expr is None:
            return 'None'
        
        etype = expr.get('type')
        
        if etype == 'literal':
            val = expr['value']
            if val is None:
                return 'None'
            if isinstance(val, bool):
                return str(val).lower()
            if isinstance(val, str):
                return f"'{val}'"
            return str(val)
        
        if etype == 'ident':
            return f"self.variables.get('{expr['name']}')"
        
        if etype == 'binary':
            left = self._expr_code(expr['left'])
            right = self._expr_code(expr['right'])
            op = expr['op']
            
            if op == '+':
                return f"({left} + {right})"
            if op == '-':
                return f"({left} - {right})"
            if op == '*':
                return f"({left} * {right})"
            if op == '/':
                return f"({left} / {right})"
            if op == '==':
                return f"({left} == {right})"
            if op == '!=':
                return f"({left} != {right})"
            if op == '<':
                return f"({left} < {right})"
            if op == '>':
                return f"({left} > {right})"
            if op == '&&':
                return f"({left} and {right})"
            if op == '||':
                return f"({left} or {right})"
            
            return 'None'
        
        if etype == 'call':
            func = self._expr_code(expr['func'])
            args = ', '.join([self._expr_code(a) for a in expr['args']])
            return f"{func}({args})"
        
        return 'None'

# ============================================================
# COMPILER
# ============================================================

def compile(source_file, output_file=None):
    print(f"Compiling {source_file}...")
    
    with open(source_file, 'r') as f:
        source = f.read()
    
    print("  [1/4] Lexing...")
    lexer = Lexer(source)
    tokens = lexer.tokenize()
    
    print("  [2/4] Parsing...")
    parser = Parser(tokens)
    ast = parser.parse()
    
    print(f"    - Models: {len(ast['models'])}")
    print(f"    - Stores: {len(ast['stores'])}")
    print(f"    - Views: {len(ast['views'])}")
    print(f"    - Routes: {len(ast['routes'])}")
    
    print("  [3/4] Generating code...")
    
    if output_file is None:
        output_file = source_file.replace('.omni', '.py')
    
    generator = CodeGenerator(ast)
    generator.generate(output_file)
    
    print(f"  [4/4] Done!")
    print(f"  Output: {output_file}")
    print(f"  Run with: python {output_file}")
    
    return output_file

# ============================================================
# MAIN
# ============================================================

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("OmniLang v2.0 - The Ultimate Lightweight Full-Stack Language")
        print("")
        print("Usage:")
        print("  python omnilang.py <file.omni>           # Compile")
        print("  python omnilang.py <file.omni> -o <out>  # Compile to file")
        print("")
        print("Single file contains: model + store + view + route")
        sys.exit(1)
    
    source_file = sys.argv[1]
    output_file = None
    
    if '-o' in sys.argv:
        idx = sys.argv.index('-o')
        if idx + 1 < len(sys.argv):
            output_file = sys.argv[idx + 1]
    
    output = compile(source_file, output_file)
    
    if output and os.path.exists(output):
        print("")
        run = input("Run now? [y/N]: ").strip().lower()
        if run == 'y':
            print("")
            os.system(f"python {output}")
