use koicore::parser::{BufReadWrapper, Parser, ParserConfig, StringInputSource};
use koicore::writer::{
    Anchor, ApplyMode, Expected, Patch, PatchError, PatchWarning, apply_patch,
    apply_patch_with_config,
};
use koicore::{Command, Parameter, Writer, WriterConfig};

fn cmd(name: &str, params: Vec<&str>) -> Command {
    Command::new(name, params.into_iter().map(Parameter::from).collect())
}

fn parse_all(source: &str) -> Vec<Command> {
    let mut parser = Parser::new(StringInputSource::new(source), ParserConfig::default());
    let mut commands = Vec::new();
    while let Some((command, _)) = parser.next_command_with_source().unwrap() {
        commands.push(command);
    }
    commands
}

#[test]
fn patches_existing_document_by_command_index() {
    let source = "#character Alice \"Hello\"\n\n## scene transition\n#background Forest\n#character Bob \"Hi\"\n";
    let mut patch = Patch::new();
    patch.delete_command(1);
    patch.insert_after_command(2, vec![cmd("action", vec!["walk"])]);
    patch.replace_command(
        3,
        vec![Command::new(
            "character",
            vec![Parameter::from("Bob"), Parameter::from("Hello there")],
        )],
    );

    let expected = "#character Alice \"Hello\"\n\n#background Forest\n#action walk\n#character Bob \"Hello there\"\n";
    assert_eq!(apply_patch(source, &patch).unwrap(), expected);
}

#[test]
fn untouched_lines_are_byte_identical() {
    let source = "  #a 1\r\n\r\n#b 2\r\n#c 3\r\n";
    let mut patch = Patch::new();
    patch.replace_line(3, vec![cmd("b", vec!["changed"])]);

    let output = apply_patch(source, &patch).unwrap();
    assert_eq!(output, "  #a 1\r\n\r\n#b changed\r\n#c 3\r\n");
}

#[test]
fn preserves_crlf_and_bom_end_to_end() {
    let source = "\u{feff}#a 1\r\n#b 2\r\n";
    let mut patch = Patch::new();
    patch.replace_line(2, vec![Command::new("c", vec![Parameter::from(3)])]);
    assert_eq!(
        apply_patch(source, &patch).unwrap(),
        "\u{feff}#a 1\r\n#c 3\r\n"
    );
}

#[test]
fn stale_line_expectation_is_rejected() {
    let source = "#a\n";
    let mut patch = Patch::new();
    patch.replace_line_checked(
        1,
        Expected::Text("#old".to_string()),
        vec![cmd("b", vec![])],
    );

    let error = apply_patch(source, &patch).unwrap_err();
    assert!(matches!(
        error,
        PatchError::AnchorMismatch { lineno: 1, .. }
    ));
}

#[test]
fn lenient_mode_skips_invalid_edits_and_warns() {
    let source = "#a\n#b\n";
    let mut patch = Patch::new();
    patch.options_mut().mode = ApplyMode::Lenient;
    patch.replace_line(9, vec![cmd("x", vec![])]);
    patch.delete_line(2);

    let mut buffer = Vec::new();
    let report = {
        let mut writer = Writer::new(&mut buffer, WriterConfig::default());
        writer.write_patched(source, &patch).unwrap()
    };

    assert_eq!(String::from_utf8(buffer).unwrap(), "#a\n");
    assert_eq!(
        report.warnings,
        vec![PatchWarning::OutOfRange { edit_index: 0 }]
    );
}

#[test]
fn patched_output_parses_back_to_expected_commands() {
    let source = "#cmd a\n#cmd b\n";
    let mut patch = Patch::new();
    patch.replace_line(2, vec![Command::new("cmd", vec![Parameter::from(2)])]);
    patch.append(vec![Command::new_text("tail")]);

    let output = apply_patch(source, &patch).unwrap();

    let commands = parse_all(&output);

    assert_eq!(
        commands,
        vec![
            Command::new("cmd", vec![Parameter::from("a")]),
            Command::new("cmd", vec![Parameter::from(2)]),
            Command::new_text("tail"),
        ]
    );
}

#[test]
fn append_to_document_without_trailing_newline_is_parseable() {
    let source = "#a";
    let mut patch = Patch::new();
    patch.append(vec![cmd("b", vec![])]);
    let output = apply_patch(source, &patch).unwrap();
    assert_eq!(output, "#a\n#b");

    let commands = parse_all(&output);
    assert_eq!(commands, vec![cmd("a", vec![]), cmd("b", vec![])]);
}

#[test]
fn command_anchor_with_expectation_guards_drift() {
    let source = "#a\n#b\n";
    let mut patch = Patch::new();
    patch.edit(
        Anchor::Command {
            index: 1,
            expect: Some(Command::new("b", Vec::new())),
        },
        koicore::writer::Edit::Delete,
    );
    assert_eq!(apply_patch(source, &patch).unwrap(), "#a\n");

    let mut drift_patch = Patch::new();
    drift_patch.edit(
        Anchor::Command {
            index: 1,
            expect: Some(Command::new("other", Vec::new())),
        },
        koicore::writer::Edit::Delete,
    );
    let error = apply_patch(source, &drift_patch).unwrap_err();
    assert!(matches!(
        error,
        PatchError::CommandMismatch { index: 1, .. }
    ));
}

#[test]
fn write_patched_matches_apply_patch_with_config() {
    let source = "    #a\n\r\n#b\n";
    let mut patch = Patch::new();
    patch.insert_after_line(1, vec![cmd("nested", vec![])]);
    patch.replace_line(3, vec![cmd("b", vec!["x"])]);

    let expected = "    #a\n    #nested\n\r\n#b x\n";
    assert_eq!(
        apply_patch_with_config(source, &patch, &WriterConfig::default()).unwrap(),
        expected
    );

    let mut buffer = Vec::new();
    {
        let mut writer = Writer::new(&mut buffer, WriterConfig::default());
        writer.write_patched(source, &patch).unwrap();
    }
    assert_eq!(String::from_utf8(buffer).unwrap(), expected);
}

#[test]
fn crlf_raw_text_newlines_reparse_with_bufread_wrapper() {
    use std::io::Cursor;

    let source = "#a\r\n";
    let mut patch = Patch::new();
    patch.replace_line(1, vec![Command::new_text("Hello\nWorld")]);
    let output = apply_patch(source, &patch).unwrap();
    assert_eq!(output, "Hello\\\r\nWorld\r\n");

    let mut parser = Parser::new(
        BufReadWrapper(Cursor::new(output.clone().into_bytes())),
        ParserConfig::default(),
    );
    let (command, _) = parser.next_command_with_source().unwrap().unwrap();
    assert_eq!(command, Command::new_text("Hello\nWorld"));
}

#[test]
fn multi_command_insert_round_trips() {
    let source = "#a\n#b\n";
    let mut patch = Patch::new();
    patch.insert_after_line(
        1,
        vec![
            Command::new("cmd", vec![Parameter::from(1)]),
            Command::new_text("hi"),
            Command::new_annotation("note"),
            Command::new_number(7, Vec::new()),
        ],
    );

    let output = apply_patch(source, &patch).unwrap();

    let commands = parse_all(&output);

    assert_eq!(
        commands,
        vec![
            cmd("a", vec![]),
            Command::new("cmd", vec![Parameter::from(1)]),
            Command::new_text("hi"),
            Command::new_annotation("note"),
            Command::new_number(7, Vec::new()),
            cmd("b", vec![]),
        ]
    );
}
