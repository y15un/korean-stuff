use std::convert::TryFrom;
use unicode_korean_multitool::{Choseong, Jongseong, Jungseong, Syllable};

// how to interpret (jongseong_a, jongseong_b, choseong_c, extended)
// => when the current syllable has `jongseong_a` and the next syllable has `Choseong::Ieung`,
//    replace the current syllable's jongseong with `jongseong_b`
//    and replace the next syllable's choseong with `choseong_c`.
// => when `extended` is true, it's part of the extended ruleset, which violates
//    phonetic equivalence.
//
// additional extended ruleset:
//     if the current syllable has either `Jongseong::Tikeut` or `Jongseong::Thikeuth`,
//     and the next syllable has any of the following vowels as jungseong:
//     * Jungseong::Ya,
//     * Jungseong::Yae,
//     * Jungseong::Yeo,
//     * Jungseong::Ye,
//     * Jungseong::Yo,
//     * Jungseong::Yu,
//     * Jungseong::I,
//     then only apply jongseong pushdown if and only if extended rulset is active.
const RULESET: [(Jongseong, Option<Jongseong>, Choseong, bool); 26] = [
    (Jongseong::Kiyeok, None, Choseong::Kiyeok, false),
    (Jongseong::SsangKiyeok, None, Choseong::SsangKiyeok, false),
    (
        Jongseong::KiyeokSios,
        Some(Jongseong::Kiyeok),
        Choseong::Sios,
        false,
    ),
    (Jongseong::Nieun, None, Choseong::Nieun, false),
    (
        Jongseong::NieunCieuc,
        Some(Jongseong::Nieun),
        Choseong::Cieuc,
        false,
    ),
    (
        Jongseong::NieunHieuh,
        Some(Jongseong::Nieun),
        Choseong::Hieuh,
        true,
    ),
    (Jongseong::Tikeut, None, Choseong::Tikeut, false),
    (Jongseong::Rieul, None, Choseong::Rieul, false),
    (
        Jongseong::RieulKiyeok,
        Some(Jongseong::Rieul),
        Choseong::Kiyeok,
        false,
    ),
    (
        Jongseong::RieulMieum,
        Some(Jongseong::Rieul),
        Choseong::Mieum,
        false,
    ),
    (
        Jongseong::RieulPieup,
        Some(Jongseong::Rieul),
        Choseong::Pieup,
        false,
    ),
    (
        Jongseong::RieulSios,
        Some(Jongseong::Rieul),
        Choseong::Sios,
        false,
    ),
    (
        Jongseong::RieulThieuth,
        Some(Jongseong::Rieul),
        Choseong::Thieuth,
        false,
    ),
    (
        Jongseong::RieulPhieuph,
        Some(Jongseong::Rieul),
        Choseong::Phieuph,
        false,
    ),
    (
        Jongseong::RieulHieuh,
        Some(Jongseong::Rieul),
        Choseong::Hieuh,
        true,
    ),
    (Jongseong::Mieum, None, Choseong::Mieum, false),
    (Jongseong::Pieup, None, Choseong::Pieup, false),
    (
        Jongseong::PieupSios,
        Some(Jongseong::Pieup),
        Choseong::Sios,
        false,
    ),
    (Jongseong::Sios, None, Choseong::Sios, false),
    (Jongseong::SsangSios, None, Choseong::SsangSios, false),
    (Jongseong::Cieuc, None, Choseong::Cieuc, false),
    (Jongseong::Chieuch, None, Choseong::Chieuch, false),
    (Jongseong::Khieukh, None, Choseong::Khieukh, false),
    (Jongseong::Thieuth, None, Choseong::Thieuth, false),
    (Jongseong::Phieuph, None, Choseong::Phieuph, false),
    (Jongseong::Hieuh, None, Choseong::Hieuh, true),
];

pub fn pushdown_jongseong(source: &str) -> String {
    pushdown_jongseong_config(source, false)
}

pub fn pushdown_jongseong_config(source: &str, extended_flag: bool) -> String {
    let mut destination = String::with_capacity(source.len());

    let mut buffer: [u8; 4] = [0, 0, 0, 0];
    let mut characters = source.chars().peekable();
    let mut new_choseong = None;

    while let Some(current) = characters.next() {
        if !Syllable::is_one_of_us(current) {
            destination.push_str(current.encode_utf8(&mut buffer));

            continue;
        }
        let mut current_syllable = Syllable::try_from(current).unwrap();
        if new_choseong.is_some() {
            current_syllable.choseong = new_choseong.take().unwrap();
        }

        if let Some(&next) = characters.peek() {
            if !Syllable::is_one_of_us(next) {
                destination.push_str(char::from(current_syllable).encode_utf8(&mut buffer));

                continue;
            }
            let next_syllable = Syllable::try_from(next).unwrap();

            // additional extended ruleset check
            if !([Some(Jongseong::Tikeut), Some(Jongseong::Thieuth)]
                .contains(&current_syllable.jongseong)
                && [
                    Jungseong::Ya,
                    Jungseong::Yae,
                    Jungseong::Yeo,
                    Jungseong::Ye,
                    Jungseong::Yo,
                    Jungseong::Yu,
                    Jungseong::I,
                ]
                .contains(&next_syllable.jungseong))
                || extended_flag
            {
                for &(
                    current_jongseong_match,
                    current_jongseong_to_be,
                    next_choseong_to_be,
                    is_extended,
                ) in RULESET.iter()
                {
                    if Some(current_jongseong_match) == current_syllable.jongseong
                        && Choseong::Ieung == next_syllable.choseong
                        && (is_extended <= extended_flag)
                    {
                        current_syllable.jongseong = current_jongseong_to_be;
                        new_choseong = Some(next_choseong_to_be);

                        break;
                    }
                }
            }
        }

        destination.push_str(char::from(current_syllable).encode_utf8(&mut buffer));
    }

    destination
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pushdown_jongseong() {
        assert_eq!(
            super::pushdown_jongseong("?????? ?????? ??????"),
            "?????? ?????? ??????".to_owned()
        );
        assert_eq!(
            super::pushdown_jongseong("?????? ?????? ?????????!"),
            "?????? ?????? ?????????!".to_owned()
        );
        assert_eq!(
            super::pushdown_jongseong("?????? ?????? ?????????!"),
            "?????? ?????? ?????????!".to_owned()
        );
        assert_eq!(
            super::pushdown_jongseong("???????????? ?????????!"),
            "???????????? ?????????!".to_owned()
        );
        assert_eq!(
            super::pushdown_jongseong("??? ????????? ??????????????? ????????????"),
            "??? ????????? ??????????????? ????????????".to_owned()
        );
        assert_eq!(
            super::pushdown_jongseong("??? ????????? ??????????????? ????????????"),
            "??? ????????? ??????????????? ????????????".to_owned()
        );
        assert_eq!(
            super::pushdown_jongseong("????????? ?????? ????????? ??????"),
            "????????? ?????? ????????? ??????".to_owned()
        );

        assert_eq!(
            super::pushdown_jongseong_config("?????? ?????? ?????????!", true),
            "?????? ?????? ?????????!".to_owned()
        );
        assert_eq!(
            super::pushdown_jongseong_config("??? ????????? ??????????????? ????????????", true),
            "??? ????????? ??????????????? ????????????".to_owned()
        );
        assert_eq!(
            super::pushdown_jongseong_config("????????? ?????? ????????? ??????", true),
            "????????? ?????? ????????? ??????".to_owned()
        );
    }
}
