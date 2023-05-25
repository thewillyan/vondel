use vondel::inter::repl;

const WELCOME_MESSAGE: &str = "
\t██╗   ██╗ ██████╗ ███╗   ██╗██████╗ ███████╗██╗     
\t██║   ██║██╔═══██╗████╗  ██║██╔══██╗██╔════╝██║     
\t██║   ██║██║   ██║██╔██╗ ██║██║  ██║█████╗  ██║     
\t╚██╗ ██╔╝██║   ██║██║╚██╗██║██║  ██║██╔══╝  ██║     
\t ╚████╔╝ ╚██████╔╝██║ ╚████║██████╔╝███████╗███████╗
\t  ╚═══╝   ╚═════╝ ╚═╝  ╚═══╝╚═════╝ ╚══════╝╚══════╝
\t                                                    
Welcome to the Vondel interpreter!
Start writing anything!
";
fn main() {
    print!("{WELCOME_MESSAGE}");
    repl::start();
}
