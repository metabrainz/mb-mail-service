use mrml::fragment::Fragment;
use mrml::mj_body::MjBodyChild;
use mrml::mj_head::MjHeadChild;
use mrmx::view;
use mrmx::WithAttribute;

pub fn head() -> Fragment<MjHeadChild> {
    view!(
        <>
            <mj-font name="Inter" href="https://fonts.googleapis.com/css?family=Inter" />

            <mj-attributes>
            <mj-all padding="10px 30px" />
                <mj-text font-size="14px" line-height="16.94px" font-weight="400" font-family="Inter" />
                <mj-class name="wrapper" border-radius="8px" background-color="#F5F5F5" padding="10px 15px" />
            </mj-attributes>
            <mj-style inline="inline">"
                h2 {
                    font-size: 12px;
                    font-weight: 700;
                }
                p {
                    margin: 6px 0;
                }
                ul {
                    padding-left: 20px;
                }
            "</mj-style>
        </>
    )
}
pub fn header() -> mrml::fragment::Fragment<MjBodyChild> {
    view!(
        <>
            <mj-image width="120px" align="left" padding="10px 15px 0px" src="https://static.metabrainz.org/logos/MusicBrainz_logo_mini.png" alt="MusicBrainz"></mj-image>

            <mj-divider padding="10px 15px" border-color="#BA478F" border-width="3px" />
        </>
    )
}
pub fn lb_header() -> mrml::fragment::Fragment<MjBodyChild> {
    view!(
        <>
            <mj-image width="120px" align="left" padding="10px 15px 0px" src="https://listenbrainz.org/static/img/listenbrainz-logo.png" alt="ListenBrainz"></mj-image>

            <mj-divider padding="10px 15px" border-color="#353070" border-width="3px" />
        <>

    )
}
