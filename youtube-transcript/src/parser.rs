use crate::error;
use crate::utils::to_human_readable;
use roxmltree::Document;
use serde;
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::error::Error;
use std::time::Duration;
#[derive(Deserialize)]
pub(crate) struct Caption {
    #[serde(rename(deserialize = "baseUrl"))]
    pub base_url: String,
    #[serde(rename(deserialize = "languageCode"))]
    pub lang_code: String,
}

#[derive(Deserialize)]
struct Captions {
    #[serde(rename(deserialize = "captionTracks"))]
    caption_tracks: Vec<Caption>,
}

pub(crate) trait HTMLParser<'a> {
    fn html_string(&'a self) -> &'a str;

    fn caption(&'a self, from: &str, to: &str, lang_code: &str) -> Result<Caption, error::Error> {
        let html = self.html_string();
        let start = html
            .split_once(from)
            .ok_or_else(|| error::Error::ParseError(format!("Cannot parse html for: {}", from)))?
            .1;
        let actual_json = start
            .split_once(to)
            .ok_or_else(|| error::Error::ParseError(format!("Cannot parse html to: {}", to)))?
            .0;
        let value: Captions = serde_json::from_str(actual_json)
            .map_err(|x| error::Error::ParseError(format!("{}", x)))?;
        let caption = value
            .caption_tracks
            .into_iter()
            .filter(|x| x.lang_code == lang_code)
            .next()
            .ok_or(error::Error::ParseError(format!(
                "Cannot find lang {lang_code}"
            )))?;
        Ok(caption)
    }
}

impl<'a> HTMLParser<'a> for String {
    fn html_string(&'a self) -> &'a str {
        self.as_str()
    }
}

impl<'a> HTMLParser<'a> for str {
    fn html_string(&'a self) -> &'a str {
        self
    }
}

/// Struct that contains data about transcirpt text along with start and duration in the whole video.
#[derive(PartialEq, Debug, Serialize)]
pub struct TranscriptCore {
    /// transcript text. Ex: "Hi How are you"
    pub text: String,
    /// starting time of the text in the whole video. Ex: "0 sec"
    pub start: Duration,
    /// duration of the text Ex: "0.8 sec"
    pub duration: Duration,
}

/// Struct containing youtube's transcript data as a Vec<[`TranscriptCore`]>
#[derive(Serialize)]
pub struct Transcript {
    /// List of transcript texts in [`TranscriptCore`] format
    pub transcripts: Vec<TranscriptCore>,
}

impl IntoIterator for Transcript {
    type IntoIter = <Vec<TranscriptCore> as IntoIterator>::IntoIter;
    type Item = TranscriptCore;

    fn into_iter(self) -> Self::IntoIter {
        self.transcripts.into_iter()
    }
}

impl From<Transcript> for String {
    fn from(value: Transcript) -> Self {
        {
            value
                .transcripts
                .into_iter()
                .map(|x| {
                    let start_h = to_human_readable(&x.start);
                    let dur_h = to_human_readable(&x.duration);
                    format!(
                        "\nstart at: {} for duration {}\n{}\n==========\n\n",
                        start_h, dur_h, x.text
                    )
                })
                .collect::<String>()
        }
    }
}

pub(crate) struct TranscriptParser;

impl TranscriptParser {
    pub fn parse<'input>(
        transcript: &'input Document<'input>,
    ) -> Result<Transcript, Box<dyn Error>> {
        let mut transcripts = Vec::new();
        let nodes = transcript
            .descendants()
            .filter(|x| x.tag_name() == "text".into());
        for node in nodes {
            let start = node
                .attribute("start")
                .ok_or(error::Error::ParseError("transcript parse error".into()))?
                .parse::<f32>()?;
            let duration = node
                .attribute("dur")
                .ok_or(error::Error::ParseError("transcript parse error".into()))?
                .parse::<f32>()?;
            let node = node
                .last_child()
                .ok_or(error::Error::ParseError("transcript parse error".into()))?;
            let text = node
                .text()
                .ok_or(error::Error::ParseError("transcript error".into()))?;

            transcripts.push(TranscriptCore {
                text: text.into(),
                start: Duration::from_secs_f32(start),
                duration: Duration::from_secs_f32(duration),
            })
        }
        Ok(Transcript { transcripts })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Config;
    const TRANSCRIPT: &str = r#"<?xml version="1.0" encoding="utf-8" ?><transcript><text start="0" dur="1.54">Hey, how&amp;#39;s it going Dave 2d here?</text><text start="1.54" dur="4.16">This is a Microsoft Surface go and when they first announced it I was interested in it</text><text start="5.7" dur="3.239">It seemed like a pretty compelling device having used it for a little while</text><text start="8.94" dur="7.29">I really think this is seriously the best product that Microsoft has put out in a very long time this thing starts at $400</text><text start="16.23" dur="4.229">I don&amp;#39;t think that base configuration is where you want to spend the money though. They have a mid tier one</text><text start="21.16" dur="5.029">550 quite a bit more but you&amp;#39;re getting double the RAM double the storage but significantly faster storage</text><text start="26.26" dur="4.49">That is the model that I think most people should pick up if you can afford that price bump</text><text start="30.75" dur="3.299">so this unit here, is that mid tier model the</text><text start="34.78" dur="2">$550 unit and I</text><text start="37.42" dur="5.209">Really like it. Ok, let&amp;#39;s go around. This thing build quality is great. It&amp;#39;s a surface product</text><text start="42.629" dur="3.21">It has a magnesium enclosure fit and finish on this is really well done</text><text start="45.84" dur="0.64">the</text><text start="46.48" dur="4.309">Top surface has these new rounded edges and it actually makes the device a lot more comfortable to hold</text><text start="50.91" dur="3.059">Not that the original surface products are like uncomfortable</text><text start="54.25" dur="5.419">But this small detail just makes it that much more ergonomic and that much more inviting to use</text><text start="59.739" dur="3.319">It&amp;#39;s a nice touch and I think Microsoft should put this kind of</text><text start="63.219" dur="4.67">Rounded edge on all of their products because it does make a difference. The screen is a 10 inch screen</text><text start="67.89" dur="1.89">I thought I&amp;#39;d be a little bit small for what I do</text><text start="69.78" dur="5.97">But it actually isn&amp;#39;t it is noticeably smaller compared to like a 12 or 13 inch screen, but it doesn&amp;#39;t feel particularly cramped</text><text start="75.75" dur="4.919">It&amp;#39;s still a very usable surface area the bezels around that screen though are thick now visually</text><text start="80.67" dur="3.33">It&amp;#39;s not attractive right having thick bezels. Like this doesn&amp;#39;t look good</text><text start="84" dur="3.839">But when you&amp;#39;re actually using it, you won&amp;#39;t notice it you&amp;#39;ll be focused on your content</text><text start="87.84" dur="3.15">it&amp;#39;s just that when this devices off or it&amp;#39;s just sitting there and you&amp;#39;re kind of</text><text start="91.21" dur="5.989">examining it visually the bezels are thick the panel itself is nice its sharp great colors and brightness and</text><text start="97.36" dur="2.599">Hitting a price point like this with this kind of screen</text><text start="99.96" dur="5.069">It could not have been easy. Like we see four or five hundred dollar devices out there that have terrible screens</text><text start="105.13" dur="4.879">This thing looks really good. There is pen support as usual and feels relatively lag free to me</text><text start="110.009" dur="0.9">I&amp;#39;m not an artist</text><text start="110.909" dur="5.22">But the surface area feels reasonably sized for people that want to use it for any kind of digital creative work</text><text start="116.35" dur="3.769">Now on the side are two speakers and they sound really good for this kind of device size</text><text start="120.219" dur="1.47">nice body to the sound</text><text start="121.689" dur="5.36">Excellent stereo separation just from the positioning and you just get really clean audio that gets to a decent volume</text><text start="127.119" dur="5.57">You also get a killer killer webcam $400 gets your webcam of this quality</text><text start="132.69" dur="3.389">it&amp;#39;s actually one of the best kans I&amp;#39;ve seen on any laptop period but when you compare</text><text start="136.39" dur="3.86">This webcam to something like a 12-inch MacBook. It just blows my mind</text><text start="140.25" dur="5.339">I mean if you can stick a webcam like this into a $400 device, there&amp;#39;s no excuse for other people</text><text start="145.59" dur="5.37">They should be using really good webcams and no one else is doing it, but surface does so good for them</text><text start="151.78" dur="6.32">this device though is not complete without the keyboard and the keyboard is a</text><text start="158.95" dur="6.68">Hundred bucks, which is crazy expensive you think about it. That&amp;#39;s like at the base model. That&amp;#39;s 20% of the cost, but</text><text start="166.6" dur="6.769">That&amp;#39;s what we have. Okay, when it&amp;#39;s connected up and it connects magnetically. It is an awesome. Awesome</text><text start="174.04" dur="0.68">productivity device</text><text start="174.72" dur="5.459">So I was concerned that this keyboard would be really small and cramped and just kind of weird feeling because it is a lot smaller</text><text start="180.18" dur="5.43">Than the regular service devices. It&amp;#39;s not cramped. It&amp;#39;s excellent. It does take a little bit of time to get used to it</text><text start="185.61" dur="5.49">But it is a really comfortable keyboard the trackpad feels good. It&amp;#39;s a surface product</text><text start="191.1" dur="5.309">So tracking is accurate and gestures work nicely. But the pad is a little small. Maybe it&amp;#39;s a visual thing</text><text start="196.41" dur="5.279">I just wish there&amp;#39;s a little bit more surface area to this trackpad. Okay performance on this device is</text><text start="202.209" dur="3.65">Good, it&amp;#39;s not amazing. It&amp;#39;s a Pentium Gold chip and most productivity stuff</text><text start="205.86" dur="4.44">Like emails web browsing or any kind of work-related stuff runs really smoothly on this</text><text start="210.31" dur="6.02">So the drive feeds on the mid-tier model actually really good fast read speed but on the slower drive of the base model the whole</text><text start="216.33" dur="4.47">System is gonna feel a bit more sluggish and that reason alone makes it worth it to upgrade to the mid-tier model</text><text start="221.2" dur="3.889">Battery life is also pretty good getting around seven hours of battery life and to charge it</text><text start="225.09" dur="3.96">You can either use the included surface connect adapter or you can use the USB C port</text><text start="229.05" dur="7.199">I really wish that the included adapter was USB C but its surface connect because well, that&amp;#39;s Microsoft&amp;#39;s for you</text><text start="236.47" dur="3.049">Okay gaming performance. I was actually surprised by this</text><text start="239.519" dur="2.31">You&amp;#39;re not gonna be able to play some killer triple-a titles</text><text start="241.83" dur="3.72">But light games are pretty good on this thing if you want to pick it up for some casual light games</text><text start="245.68" dur="1.22">It&amp;#39;ll do the trick now</text><text start="246.9" dur="5.07">The surface go is still a surface product through and through so if they&amp;#39;re issues you had with surface products in the past</text><text start="252.04" dur="4.669">You may have those same issues with this one. Like if you need more ports, there&amp;#39;s still only one port</text><text start="256.709" dur="1.261">It&amp;#39;s use BC this year</text><text start="257.97" dur="3.869">But it&amp;#39;s still only one port if you don&amp;#39;t like the kickstand on your lap</text><text start="261.84" dur="4.44">Like it&amp;#39;s not an ideal situation for lap use, but it does work reasonably</text><text start="266.28" dur="3.839">Well plus infinite positions up to a certain degree, but I don&amp;#39;t know</text><text start="270.12" dur="5.339">This makes it fairly usable for most people I think but if you&amp;#39;ve had issues in the past same issues now</text><text start="276.759" dur="2.93">Overall though really good product. I think for students</text><text start="279.69" dur="3.239">This is such a good option you get so much versatility on this thing</text><text start="282.93" dur="1.38">You get a great keyboard for taking notes</text><text start="284.31" dur="2.1">You can pull up course material and stuff in class</text><text start="286.63" dur="4.159">Good option great for me to consumption for like a secondary device if you want it for that</text><text start="290.789" dur="3.45">I think you can&amp;#39;t go wrong with this it is however not</text><text start="294.88" dur="4.699">Cheap once you add everything up together like the keyboard and like the mid tier unit</text><text start="299.58" dur="2.64">It&amp;#39;s not the $400 device that they&amp;#39;re kind of marketing</text><text start="302.22" dur="5.669">So you kind of have to take that into consideration, but overall I like this thing. Ok. Hope you guys enjoyed this video thumbs</text><text start="307.889" dur="2.339">We liked it subs we loved it. See you guys next time</text></transcript>"#;
    struct HTML;

    impl HTMLParser<'_> for HTML {
        fn html_string(&self) -> &'static str {
            r#""elapsedMediaTimeSeconds": 0 } }, "captions": { "playerCaptionsTracklistRenderer": { "captionTracks": [{ "baseUrl": "https://www.youtube.com/api/timedtext?v=GJLlxj_dtq8\u0026caps=asr\u0026xoaf=5\u0026hl=en-GB\u0026ip=0.0.0.0\u0026ipbits=0\u0026expire=1681082354\u0026sparams=ip,ipbits,expire,v,caps,xoaf\u0026signature=13D068D838F3B1262B96D29751914C9E75100C4C.A99B64907A100E2E5F74ACE0BA586FB82F865CE3\u0026key=yt8\u0026lang=zh", "name": { "simpleText": "Chinese" }, "vssId": ".zh", "languageCode": "zh", "isTranslatable": true }, { "baseUrl": "https://www.youtube.com/api/timedtext?v=GJLlxj_dtq8\u0026caps=asr\u0026xoaf=5\u0026hl=en-GB\u0026ip=0.0.0.0\u0026ipbits=0\u0026expire=1681082354\u0026sparams=ip,ipbits,expire,v,caps,xoaf\u0026signature=13D068D838F3B1262B96D29751914C9E75100C4C.A99B64907A100E2E5F74ACE0BA586FB82F865CE3\u0026key=yt8\u0026lang=cs", "name": { "simpleText": "Czech" }, "vssId": ".cs", "languageCode": "cs", "isTranslatable": true }, { "baseUrl": "https://www.youtube.com/api/timedtext?v=GJLlxj_dtq8\u0026caps=asr\u0026xoaf=5\u0026hl=en-GB\u0026ip=0.0.0.0\u0026ipbits=0\u0026expire=1681082354\u0026sparams=ip,ipbits,expire,v,caps,xoaf\u0026signature=13D068D838F3B1262B96D29751914C9E75100C4C.A99B64907A100E2E5F74ACE0BA586FB82F865CE3\u0026key=yt8\u0026lang=en", "name": { "simpleText": "English" }, "vssId": ".en", "languageCode": "en", "isTranslatable": true }, { "baseUrl": "https://www.youtube.com/api/timedtext?v=GJLlxj_dtq8\u0026caps=asr\u0026xoaf=5\u0026hl=en-GB\u0026ip=0.0.0.0\u0026ipbits=0\u0026expire=1681082354\u0026sparams=ip,ipbits,expire,v,caps,xoaf\u0026signature=13D068D838F3B1262B96D29751914C9E75100C4C.A99B64907A100E2E5F74ACE0BA586FB82F865CE3\u0026key=yt8\u0026kind=asr\u0026lang=en", "name": { "simpleText": "English (auto-generated)" }, "vssId": "a.en", "languageCode": "en", "kind": "asr", "isTranslatable": true }, { "baseUrl": "https://www.youtube.com/api/timedtext?v=GJLlxj_dtq8\u0026caps=asr\u0026xoaf=5\u0026hl=en-GB\u0026ip=0.0.0.0\u0026ipbits=0\u0026expire=1681082354\u0026sparams=ip,ipbits,expire,v,caps,xoaf\u0026signature=13D068D838F3B1262B96D29751914C9E75100C4C.A99B64907A100E2E5F74ACE0BA586FB82F865CE3\u0026key=yt8\u0026lang=de", "name": { "simpleText": "German" }, "vssId": ".de", "languageCode": "de", "isTranslatable": true }, { "baseUrl": "https://www.youtube.com/api/timedtext?v=GJLlxj_dtq8\u0026caps=asr\u0026xoaf=5\u0026hl=en-GB\u0026ip=0.0.0.0\u0026ipbits=0\u0026expire=1681082354\u0026sparams=ip,ipbits,expire,v,caps,xoaf\u0026signature=13D068D838F3B1262B96D29751914C9E75100C4C.A99B64907A100E2E5F74ACE0BA586FB82F865CE3\u0026key=yt8\u0026lang=hi", "name": { "simpleText": "Hindi" }, "vssId": ".hi", "languageCode": "hi", "isTranslatable": true }, { "baseUrl": "https://www.youtube.com/api/timedtext?v=GJLlxj_dtq8\u0026caps=asr\u0026xoaf=5\u0026hl=en-GB\u0026ip=0.0.0.0\u0026ipbits=0\u0026expire=1681082354\u0026sparams=ip,ipbits,expire,v,caps,xoaf\u0026signature=13D068D838F3B1262B96D29751914C9E75100C4C.A99B64907A100E2E5F74ACE0BA586FB82F865CE3\u0026key=yt8\u0026lang=ja", "name": { "simpleText": "Japanese" }, "vssId": ".ja", "languageCode": "ja", "isTranslatable": true }, { "baseUrl": "https://www.youtube.com/api/timedtext?v=GJLlxj_dtq8\u0026caps=asr\u0026xoaf=5\u0026hl=en-GB\u0026ip=0.0.0.0\u0026ipbits=0\u0026expire=1681082354\u0026sparams=ip,ipbits,expire,v,caps,xoaf\u0026signature=13D068D838F3B1262B96D29751914C9E75100C4C.A99B64907A100E2E5F74ACE0BA586FB82F865CE3\u0026key=yt8\u0026lang=ko", "name": { "simpleText": "Korean" }, "vssId": ".ko", "languageCode": "ko", "isTranslatable": true }, { "baseUrl": "https://www.youtube.com/api/timedtext?v=GJLlxj_dtq8\u0026caps=asr\u0026xoaf=5\u0026hl=en-GB\u0026ip=0.0.0.0\u0026ipbits=0\u0026expire=1681082354\u0026sparams=ip,ipbits,expire,v,caps,xoaf\u0026signature=13D068D838F3B1262B96D29751914C9E75100C4C.A99B64907A100E2E5F74ACE0BA586FB82F865CE3\u0026key=yt8\u0026lang=es", "name": { "simpleText": "Spanish" }, "vssId": ".es", "languageCode": "es", "isTranslatable": true }], "audioTracks": [{ "captionTrackIndices": [0, 1, 2, 4, 5, 6, 7, 8, 3], "defaultCaptionTrackIndex": 2, "visibility": "UNKNOWN", "hasDefaultTrack": true, "captionsInitialState": "CAPTIONS_INITIAL_STATE_OFF_RECOMMENDED" }], "translationLanguages": [{ "languageCode": "af", "languageName": { "simpleText": "Afrikaans" } }, { "languageCode": "ak", "languageName": { "simpleText": "Akan" } }, { "languageCode": "sq", "languageName": { "simpleText": "Albanian" } }, { "languageCode": "am", "languageName": { "simpleText": "Amharic" } }, { "languageCode": "ar", "languageName": { "simpleText": "Arabic" } }, { "languageCode": "hy", "languageName": { "simpleText": "Armenian" } }, { "languageCode": "as", "languageName": { "simpleText": "Assamese" } }, { "languageCode": "ay", "languageName": { "simpleText": "Aymara" } }, { "languageCode": "az", "languageName": { "simpleText": "Azerbaijani" } }, { "languageCode": "bn", "languageName": { "simpleText": "Bangla" } }, { "languageCode": "eu", "languageName": { "simpleText": "Basque" } }, { "languageCode": "be", "languageName": { "simpleText": "Belarusian" } }, { "languageCode": "bho", "languageName": { "simpleText": "Bhojpuri" } }, { "languageCode": "bs", "languageName": { "simpleText": "Bosnian" } }, { "languageCode": "bg", "languageName": { "simpleText": "Bulgarian" } }, { "languageCode": "my", "languageName": { "simpleText": "Burmese" } }, { "languageCode": "ca", "languageName": { "simpleText": "Catalan" } }, { "languageCode": "ceb", "languageName": { "simpleText": "Cebuano" } }, { "languageCode": "zh-Hans", "languageName": { "simpleText": "Chinese (Simplified)" } }, { "languageCode": "zh-Hant", "languageName": { "simpleText": "Chinese (Traditional)" } }, { "languageCode": "co", "languageName": { "simpleText": "Corsican" } }, { "languageCode": "hr", "languageName": { "simpleText": "Croatian" } }, { "languageCode": "cs", "languageName": { "simpleText": "Czech" } }, { "languageCode": "da", "languageName": { "simpleText": "Danish" } }, { "languageCode": "dv", "languageName": { "simpleText": "Divehi" } }, { "languageCode": "nl", "languageName": { "simpleText": "Dutch" } }, { "languageCode": "en", "languageName": { "simpleText": "English" } }, { "languageCode": "eo", "languageName": { "simpleText": "Esperanto" } }, { "languageCode": "et", "languageName": { "simpleText": "Estonian" } }, { "languageCode": "ee", "languageName": { "simpleText": "Ewe" } }, { "languageCode": "fil", "languageName": { "simpleText": "Filipino" } }, { "languageCode": "fi", "languageName": { "simpleText": "Finnish" } }, { "languageCode": "fr", "languageName": { "simpleText": "French" } }, { "languageCode": "gl", "languageName": { "simpleText": "Galician" } }, { "languageCode": "lg", "languageName": { "simpleText": "Ganda" } }, { "languageCode": "ka", "languageName": { "simpleText": "Georgian" } }, { "languageCode": "de", "languageName": { "simpleText": "German" } }, { "languageCode": "el", "languageName": { "simpleText": "Greek" } }, { "languageCode": "gn", "languageName": { "simpleText": "Guarani" } }, { "languageCode": "gu", "languageName": { "simpleText": "Gujarati" } }, { "languageCode": "ht", "languageName": { "simpleText": "Haitian Creole" } }, { "languageCode": "ha", "languageName": { "simpleText": "Hausa" } }, { "languageCode": "haw", "languageName": { "simpleText": "Hawaiian" } }, { "languageCode": "iw", "languageName": { "simpleText": "Hebrew" } }, { "languageCode": "hi", "languageName": { "simpleText": "Hindi" } }, { "languageCode": "hmn", "languageName": { "simpleText": "Hmong" } }, { "languageCode": "hu", "languageName": { "simpleText": "Hungarian" } }, { "languageCode": "is", "languageName": { "simpleText": "Icelandic" } }, { "languageCode": "ig", "languageName": { "simpleText": "Igbo" } }, { "languageCode": "id", "languageName": { "simpleText": "Indonesian" } }, { "languageCode": "ga", "languageName": { "simpleText": "Irish" } }, { "languageCode": "it", "languageName": { "simpleText": "Italian" } }, { "languageCode": "ja", "languageName": { "simpleText": "Japanese" } }, { "languageCode": "jv", "languageName": { "simpleText": "Javanese" } }, { "languageCode": "kn", "languageName": { "simpleText": "Kannada" } }, { "languageCode": "kk", "languageName": { "simpleText": "Kazakh" } }, { "languageCode": "km", "languageName": { "simpleText": "Khmer" } }, { "languageCode": "rw", "languageName": { "simpleText": "Kinyarwanda" } }, { "languageCode": "ko", "languageName": { "simpleText": "Korean" } }, { "languageCode": "kri", "languageName": { "simpleText": "Krio" } }, { "languageCode": "ku", "languageName": { "simpleText": "Kurdish" } }, { "languageCode": "ky", "languageName": { "simpleText": "Kyrgyz" } }, { "languageCode": "lo", "languageName": { "simpleText": "Lao" } }, { "languageCode": "la", "languageName": { "simpleText": "Latin" } }, { "languageCode": "lv", "languageName": { "simpleText": "Latvian" } }, { "languageCode": "ln", "languageName": { "simpleText": "Lingala" } }, { "languageCode": "lt", "languageName": { "simpleText": "Lithuanian" } }, { "languageCode": "lb", "languageName": { "simpleText": "Luxembourgish" } }, { "languageCode": "mk", "languageName": { "simpleText": "Macedonian" } }, { "languageCode": "mg", "languageName": { "simpleText": "Malagasy" } }, { "languageCode": "ms", "languageName": { "simpleText": "Malay" } }, { "languageCode": "ml", "languageName": { "simpleText": "Malayalam" } }, { "languageCode": "mt", "languageName": { "simpleText": "Maltese" } }, { "languageCode": "mi", "languageName": { "simpleText": "Māori" } }, { "languageCode": "mr", "languageName": { "simpleText": "Marathi" } }, { "languageCode": "mn", "languageName": { "simpleText": "Mongolian" } }, { "languageCode": "ne", "languageName": { "simpleText": "Nepali" } }, { "languageCode": "nso", "languageName": { "simpleText": "Northern Sotho" } }, { "languageCode": "no", "languageName": { "simpleText": "Norwegian" } }, { "languageCode": "ny", "languageName": { "simpleText": "Nyanja" } }, { "languageCode": "or", "languageName": { "simpleText": "Odia" } }, { "languageCode": "om", "languageName": { "simpleText": "Oromo" } }, { "languageCode": "ps", "languageName": { "simpleText": "Pashto" } }, { "languageCode": "fa", "languageName": { "simpleText": "Persian" } }, { "languageCode": "pl", "languageName": { "simpleText": "Polish" } }, { "languageCode": "pt", "languageName": { "simpleText": "Portuguese" } }, { "languageCode": "pa", "languageName": { "simpleText": "Punjabi" } }, { "languageCode": "qu", "languageName": { "simpleText": "Quechua" } }, { "languageCode": "ro", "languageName": { "simpleText": "Romanian" } }, { "languageCode": "ru", "languageName": { "simpleText": "Russian" } }, { "languageCode": "sm", "languageName": { "simpleText": "Samoan" } }, { "languageCode": "sa", "languageName": { "simpleText": "Sanskrit" } }, { "languageCode": "gd", "languageName": { "simpleText": "Scottish Gaelic" } }, { "languageCode": "sr", "languageName": { "simpleText": "Serbian" } }, { "languageCode": "sn", "languageName": { "simpleText": "Shona" } }, { "languageCode": "sd", "languageName": { "simpleText": "Sindhi" } }, { "languageCode": "si", "languageName": { "simpleText": "Sinhala" } }, { "languageCode": "sk", "languageName": { "simpleText": "Slovak" } }, { "languageCode": "sl", "languageName": { "simpleText": "Slovenian" } }, { "languageCode": "so", "languageName": { "simpleText": "Somali" } }, { "languageCode": "st", "languageName": { "simpleText": "Southern Sotho" } }, { "languageCode": "es", "languageName": { "simpleText": "Spanish" } }, { "languageCode": "su", "languageName": { "simpleText": "Sundanese" } }, { "languageCode": "sw", "languageName": { "simpleText": "Swahili" } }, { "languageCode": "sv", "languageName": { "simpleText": "Swedish" } }, { "languageCode": "tg", "languageName": { "simpleText": "Tajik" } }, { "languageCode": "ta", "languageName": { "simpleText": "Tamil" } }, { "languageCode": "tt", "languageName": { "simpleText": "Tatar" } }, { "languageCode": "te", "languageName": { "simpleText": "Telugu" } }, { "languageCode": "th", "languageName": { "simpleText": "Thai" } }, { "languageCode": "ti", "languageName": { "simpleText": "Tigrinya" } }, { "languageCode": "ts", "languageName": { "simpleText": "Tsonga" } }, { "languageCode": "tr", "languageName": { "simpleText": "Turkish" } }, { "languageCode": "tk", "languageName": { "simpleText": "Turkmen" } }, { "languageCode": "uk", "languageName": { "simpleText": "Ukrainian" } }, { "languageCode": "ur", "languageName": { "simpleText": "Urdu" } }, { "languageCode": "ug", "languageName": { "simpleText": "Uyghur" } }, { "languageCode": "uz", "languageName": { "simpleText": "Uzbek" } }, { "languageCode": "vi", "languageName": { "simpleText": "Vietnamese" } }, { "languageCode": "cy", "languageName": { "simpleText": "Welsh" } }, { "languageCode": "fy", "languageName": { "simpleText": "Western Frisian" } }, { "languageCode": "xh", "languageName": { "simpleText": "Xhosa" } }, { "languageCode": "yi", "languageName": { "simpleText": "Yiddish" } }, { "languageCode": "yo", "languageName": { "simpleText": "Yoruba" } }, { "languageCode": "zu", "languageName": { "simpleText": "Zulu" } }], "defaultAudioTrackIndex": 0 }},"videoDetails": { "videoId": "GJLlxj_dtq8", "#
        }
    }
    #[test]
    fn test_caption() {
        let c = Config::default();
        let caption = HTML.caption(c.parser.from, c.parser.to, "en").unwrap();
        assert_eq!(caption.base_url, "https://www.youtube.com/api/timedtext?v=GJLlxj_dtq8&caps=asr&xoaf=5&hl=en-GB&ip=0.0.0.0&ipbits=0&expire=1681082354&sparams=ip,ipbits,expire,v,caps,xoaf&signature=13D068D838F3B1262B96D29751914C9E75100C4C.A99B64907A100E2E5F74ACE0BA586FB82F865CE3&key=yt8&lang=en");
    }

    #[test]
    fn test_transcript_parse() {
        let doc = Document::parse(TRANSCRIPT).unwrap();
        let parsed = TranscriptParser::parse(&doc).unwrap();
        assert_eq!(parsed.transcripts.len(), 74)
    }
    #[test]
    fn test_transcript_parse_time() {
        let doc = Document::parse(TRANSCRIPT).unwrap();
        let parsed = TranscriptParser::parse(&doc).unwrap();
        let elem = parsed.into_iter().next().unwrap();
        assert_eq!(elem.start, Duration::from_millis(0));
        assert_eq!(elem.duration, Duration::from_secs_f32(1.54))
    }
}
