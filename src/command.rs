pub fn parse_quotes(vec: &mut Vec<String>, args: String) -> bool {
    let mut esc: bool = false;
    let mut buf: String = String::new();
    let mut quote: bool = false;

    macro_rules! esc_char {
        ($cchr: expr, $pchr: expr, $chr: expr, $buf: expr, $esc: expr, $else: expr) => {
            if $chr == $cchr {
                $esc = false;
                $buf.push($pchr);
            } else {
                $else
            }
        };
    }

    for chr in args.chars() {
        if esc {
            esc_char!('\\', '\\', chr, buf, esc,
            esc_char!('n', '\n', chr, buf, esc,
            esc_char!('r', '\r', chr, buf, esc,
            esc_char!('t', '\t', chr, buf, esc,
            esc_char!('\"', '\"', chr, buf, esc,
            esc_char!(' ', ' ', chr, buf, esc,
            esc_char!('\t', '\t', chr, buf, esc,
            esc_char!('\n', '\n', chr, buf, esc,
            esc_char!('\r', '\r', chr, buf, esc,
            {
                esc = true;
                buf.push('\\');
                buf.push(chr);
            }
            )))))))));
        } else if chr == '\\' {
            esc = true;
        } else if chr == '\"' {
            quote = !quote;
        } else if [' ', '\t', '\n', '\r'].contains(&chr) {
            if quote {
                buf.push(chr);
            } else if !buf.is_empty() {
                vec.push(buf.clone());
                buf.clear();
            }
        } else {
            buf.push(chr);
        }
    };

    quote
}
