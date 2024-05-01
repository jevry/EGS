//to do the sexp stuff
use symbolic_expressions::Sexp;

//print out a Sexpr in a pretty manner
pub fn pretty_print(sexp: &Sexp, width: usize){
    let mut buf = String::new();
    format_pretty_string(&mut buf, sexp, width, 1).unwrap();
    print!("\n{}\n\n", buf);
}

//formats the string
pub fn format_pretty_string(buf: &mut String, sexp: &Sexp, width: usize, level: usize) -> std::fmt::Result {
    use std::fmt::Write;
    if let Sexp::List(list) = sexp {
        let indent = sexp.to_string().len() > width;
        write!(buf, "(")?;

        for (i, val) in list.iter().enumerate() {
            if indent && i > 0 {
                writeln!(buf)?;
                for _ in 0..level {
                    write!(buf, "   ")?;
                }
            }
            format_pretty_string(buf, val, width, level + 1)?;
            if !indent && i < list.len() - 1 {
                write!(buf, "  ")?;
            }
        }

        write!(buf, ")")?;
        Ok(())
    } else {
        write!(buf, "{}", sexp.to_string().trim_matches('"'))
    }
}
