use orca_iot::parser;

#[test]
fn test_whitespace_parser() {
    let parse = parser::whitespace;

    let cases = [' ', '\t', '\n'];

    for c in cases {
        assert_eq!(parse(&c.to_string()), Ok(("", c)));
    }
}

#[test]
fn test_value_parser() {
    let parse = parser::value;

    let cases = [
        "1000",
        "  1000",
        "1000 ",
        "\t1000",
        "1000\t",
        "\n1000",
        "1000\n",
        "  1000",
        "1000  ",
        "\n\n1000",
        "1000\n\n",
        "\t\t1000",
        "1000\t\t",
        " \t\n1000",
        "1000\t\n ",
        " \t\n1000\t\n ",
    ];

    for c in cases {
        assert_eq!(parse(c), Ok(("", 1000.0)))
    }
}

#[test]
fn test_keyword_parser() {
    let mut parse = parser::keyword("humidity");

    let cases = [
        "humidity",
        " humidity",
        "humidity ",
        " humidity ",
        "\nhumidity",
        "humidity\n",
        "\nhumidity\n",
        "\thumidity",
        "humidity\t",
        " \n\thumidity\t\n ",
    ];

    for c in cases {
        assert_eq!(parse(c), Ok(("", "humidity")));
    }
}

#[test]
fn test_reading_parser() {
    let mut parse = parser::reading("humidity");

    let cases = [
        "humidity:70",
        " humidity:70",
        "humidity :70",
        " humidity :70",
        "\nhumidity:70",
        "humidity\n:70",
        "\nhumidity\n:70",
        "\thumidity:70",
        "humidity\t:70",
        " \n\thumidity\t\n :70",
        " humidity: 70 ",
        "humidity :70 ",
        " humidity : 70 ",
        "\nhumidity:\n70",
        "humidity\n:70\n",
        "\nhumidity\n:\n70\n",
        "\thumidity:\t70",
        "humidity\t:70\t",
        " \n\thumidity\t\n : \n\t70\t\n ",
    ];

    for c in cases {
        assert_eq!(parse(c), Ok(("", ("humidity", 70.0))));
    }
}

#[test]
fn test_readings_parser() {
    let result = parser::readings(
        "

        temperature:
            34.97


        pressure:
            100384.0
        
        windspeed:
            0.0

        waterlevel:
            96.68

        humidity:
            66.0
        
        ",
    );

    assert_eq!(
        result,
        Ok((
            "",
            (
                ("temperature", 34.97),
                ("pressure", 100384.0),
                ("windspeed", 0.0),
                ("waterlevel", 96.68),
                ("humidity", 66.0),
            )
        ))
    )
}

#[test]
fn test_eof() {
    let inp = "temperature:27.77pressure:1011.78windspeed:0.00waterlevel:19.01humidity:87.00";

    assert_eq!(
        parser::consume(inp),
        Ok((
            "",
            (
                ("temperature", 27.77),
                ("pressure", 1011.78),
                ("windspeed", 0.00),
                ("waterlevel", 19.01),
                ("humidity", 87.00),
            )
        ))
    )
}
