use mrml::mj_body::MjBodyChild;
use mrmx::view;
use mrmx::WithAttribute;

pub fn header() -> mrml::fragment::Fragment<MjBodyChild> {
    view!(
        <>
            <mj-image width="120px" align="left" padding="10px 15px 0px" src="https://static.metabrainz.org/MB/header-logo-1f7dc2a.svg"></mj-image>

            <mj-divider padding="10px 15px" border-color="#BA478F" border-width="3px" />
        </>
    )
}
