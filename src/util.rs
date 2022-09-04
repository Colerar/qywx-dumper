pub trait ReplaceSpecial {
  fn replace_special_char(self) -> String;
}

impl ReplaceSpecial for String {
  fn replace_special_char(self) -> String {
    const SPECIALS: [char; 9] = ['?', '*', ':', '"', '<', '>', '\\', '/', '|'];
    #[rustfmt::skip]
    const NON_PRINTABLE: [char; 32] = [
      0u8 as char, 1u8 as char, 2u8 as char, 3u8 as char, 4u8 as char, 5u8 as char, 6u8 as char,
      7u8 as char, 8u8 as char, 9u8 as char, 10u8 as char, 11u8 as char, 12u8 as char, 13u8 as char,
      14u8 as char, 15u8 as char, 16u8 as char, 17u8 as char, 18u8 as char, 19u8 as char, 20u8 as char,
      21u8 as char, 22u8 as char, 23u8 as char, 24u8 as char, 25u8 as char, 26u8 as char, 27u8 as char,
      28u8 as char, 29u8 as char, 30u8 as char, 31u8 as char,
    ];

    let remove_special = SPECIALS
      .into_iter()
      .fold(self, |acc, i| acc.replace(i, "-"));

    NON_PRINTABLE
      .into_iter()
      .fold(remove_special, |acc, i| acc.replace(i, ""))
  }
}
