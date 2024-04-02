// install junit-platform-console.*.jar in system path
// java -jar junit-platform-console-standalone-1.10.2.jar discover -cp bin --scan-class-path
// --details testfeed
// using the results filter to only one matching test name
// then feed then into java -jar junit-platform-console-standalone-1.10.2.jar execute -cp bin --scan-class-path
// the format for exexute is class#name (i think)
// also when compiling the jar has to be in classpath
// this is what needed for junit5, (defualt)
//
// we will probably overiding this to use junit4 in Brew.toml

pub fn sip(search: Option<String>) {
    todo!()
}
