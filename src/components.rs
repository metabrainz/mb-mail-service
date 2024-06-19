use mrml::mj_body::MjBodyChild;
use mrmx::view;
use mrmx::WithAttribute;

pub fn header() -> mrml::fragment::Fragment<MjBodyChild> {
    view!(
        <>
            <mj-image width="200px" align="left" src="https://static.metabrainz.org/MB/header-logo-1f7dc2a.svg"></mj-image>

            <mj-divider border-color="#BA478F"></mj-divider>
        </>
    )
}
