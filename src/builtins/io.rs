/*
 *  Copyright (C) 2021, Wafelack <wafelack@protonmail.com>
 *
 *  ------------------------------------------------------
 *
 *     This file is part of Orion.
 *
 *  Orion is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
 *  Orion is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with Orion.  If not, see <https://www.gnu.org/licenses/>.
 */
use crate::{interpreter::{Interpreter, Value}, OrionError, error, bug, Result, parser::{Pattern, Expr, Literal}};
use std::{path::Path, fs::OpenOptions, io::Write};

impl Interpreter {
    pub fn put_str(&mut self, args: Vec<Value>) -> Result<Value> {
        print!("{}", self.get_lit_val(&args[0]));
        Ok(Value::Unit)
    }
    pub fn put_str_ln(&mut self, args: Vec<Value>) -> Result<Value> {
        println!("{}", self.get_lit_val(&args[0]));
        Ok(Value::Unit)
    }
    pub fn write(&mut self, args: Vec<Value>) -> Result<Value> {
        let file = if let Value::String(of) = &args[1] {
            of
        } else {
            return error!("Expected a String, found a {}.", self.get_val_type(&args[1]));
        };

        if !Path::new(file).exists() {
            return error!("File `{}` does not exist.", file);
        }
        
        let written = match {match OpenOptions::new().append(true).open(file) {
            Ok(f) => f,
            Err(e) => return error!("Failed to open `{}`: {}.", file, e),
        }}.write(self.get_lit_val(&args[0]).as_bytes()) {
            Ok(u) => u,
            Err(e) => return error!("Failed to write `{}`: {}.", file, e),
        };


        Ok(Value::Integer(written as i32))
    }


}
