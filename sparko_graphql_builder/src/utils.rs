pub fn to_pascal_case(s: &str) -> String {
    let mut string = String::with_capacity(s.len());

    enum Phase {
        Leadin,
        Before,
        FirstUpper,
        Upper(char),
        Lower
    }

    let mut phase = Phase::Leadin;
    let mut underscores = 0;
    let mut push = |char| string.push(char);

    for char in s.chars() {
        

        match phase {
            Phase::Leadin => {
                if char == '_' {
                    underscores += 1;
                }
                else {
                    char.to_uppercase().for_each(&mut push);
                    phase = Phase::FirstUpper;
                }
            },
            Phase::Before => {
                if char.is_alphanumeric() {
                    char.to_uppercase().for_each(&mut push);
                    phase = Phase::FirstUpper;
                }
                // else skip
            },
            Phase::FirstUpper => {
                if char.is_uppercase() {
                    phase = Phase::Upper(char);
                }
                else {
                    if char.is_alphanumeric() {
                        char.to_lowercase().for_each(&mut push);
                    }
                    phase = Phase::Lower;
                }
            },
            Phase::Upper(pending) => {

                if char.is_uppercase() {
                    pending.to_lowercase().for_each(&mut push);
                    phase = Phase::Upper(char);
                }
                else {
                    if char.is_alphanumeric() {
                        pending.to_uppercase().for_each(&mut push);
                        char.to_lowercase().for_each(&mut push);
                        phase = Phase::Lower;
                    }
                    else {
                        pending.to_lowercase().for_each(&mut push);
                        phase = Phase::Before;
                    }
                }
            },
            Phase::Lower => {
                if char.is_lowercase() {
                    push(char);
                }
                else {
                    if char.is_alphanumeric() {
                        char.to_uppercase().for_each(&mut push);
                        phase = Phase::FirstUpper;
                    }
                    else {
                        phase = Phase::Before;
                    }
                }
            },
        }
    }

    match phase {
        Phase::Leadin => string = format!("underscore{}", underscores),
        Phase::Upper(pending) => pending.to_lowercase().for_each(&mut push),
        _ => {},
    }

    string
}

// struct IndentedFormatter<'a> {
//     base: &'a mut std::fmt::Formatter<'a>,
//     indent: usize,
//     start_of_line: bool,
// }

// impl IndentedFormatter<'_> {
//     pub fn indent(&mut self) -> IndentedFormatter {
//         IndentedFormatter {
//             base: self.base,
//             indent: self.indent + 1,
//             start_of_line: true,
//         }
//     }
// }

// impl std::fmt::Write for IndentedFormatter<'_> {
//     fn write_str(&mut self, str: &str) -> std::fmt::Result {
//         if str.contains("\n") {
//             let mut t: usize = 1;
//             let mut buf = str.replace("\n","    ");
//             while(t < self.indent) {
//                 buf = buf.replace("\n","    ");
//                 t += 1;
//             }
//             self.base.write_str(str)
//         }
//         else {
//             self.base.write_str(str)
//         }
//     }

// }


// struct IndentedWriter<'a> {
//     base: &'a mut Vec<u8>,
//     indent: usize,
//     start_of_line: bool,
// }

// impl IndentedWriter<'_> {
//     pub fn indent(&mut self) -> IndentedWriter {
//         IndentedWriter {
//             base: self.base,
//             indent: self.indent + 1,
//             start_of_line: true,
//         }
//     }
// }

// impl std::io::Write for IndentedWriter<'_> {
//     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
//         let mut nbytes: usize = 0;
//         let mut s = 0;

//         while s< buf.len() {
//             if self.start_of_line {
//                 let mut t: usize = 0;

//                 while(t < self.indent) {
//                     self.base.write(b"    ")?;
//                     t += 1;
//                 }

//                 self.start_of_line = false;
//             }
//             let mut e = s;
//             while e < buf.len() && buf[e] != b'\n' {
//                 e += 1;
//             }

//             if e < buf.len() {
//                 e += 1;
//                 nbytes += self.base.write(&buf[s..e])?;

//                 self.start_of_line = true;
//             }
//             else {
//                 nbytes += self.base.write(&buf[s..e])?;
//             }
//             s = e;
//         }
//         Ok(nbytes)
//     }

//     fn flush(&mut self) -> std::io::Result<()> {
//         self.base.flush()
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pascal_case() {

        assert_eq!(to_pascal_case("HTTPServer"), "HttpServer");
        assert_eq!(to_pascal_case("HTTP"), "Http");
        assert_eq!(to_pascal_case("être"), "Être");
        assert_eq!(to_pascal_case("d'être"), "Dêtre");
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("HelloWorld"), "HelloWorld");
        assert_eq!(to_pascal_case("_hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("___"), "underscore3");
    }
}
