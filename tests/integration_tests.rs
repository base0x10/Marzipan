use marzipan::redcode;

#[test]
fn redcode_algebra() {
    let a = redcode::RedAddr::new(redcode::CORESIZE - 1);
    let b = redcode::RedAddr::from_i32(-1);
    assert_eq!(a.value(), (redcode::CORESIZE - 1));
    assert_eq!(b.value(), (redcode::CORESIZE - 1));

    assert_eq!(a.value(), b.value());

    let c = a + b;
    assert_eq!(c.value(), (redcode::CORESIZE - 2));
}

#[test]
fn imp_test() {
    /// End to end test with imp loadfile
    let _loadfile = r#"
;redcode
;name          Dwarf
;author        A. K. Dewdney
;version       94.1
;date          April 29, 1993
;strategy      Bombs every fourth instruction.
ORG     1          ; Indicates execution begins with the second
; instruction (ORG is not actually loaded, and is
; therefore not counted as an instruction).

DAT.F   #0, #0     ; Pointer to target instruction.
ADD.AB  #4, $-1    ; Increments pointer by step.
MOV.AB  #0, @-2    ; Bombs target instruction.
JMP.A   $-2, #0    ; Loops back two instructions.
"#;

    let loadfile = r#"
DAT.F      #0,     #0
ADD.AB #4, $-1
MOV.AB #0, @-2
JMP.A $-2, #0
"#;

    println!("{}", loadfile);
    let res = marzipan::parser::loadfile::parse_loadfile(&loadfile);
    println!("{:?}", res);
    assert_eq!(4, res.unwrap().0.len());
    assert_eq!(
        marzipan::parser::loadfile::parse_loadfile(&loadfile)
            .unwrap()
            .0,
        marzipan::parser::loadfile::parse_loadfile(&_loadfile)
            .unwrap()
            .0
    );
}
