use crate::lettre::LettreError;
use lettre::message::{Mailbox, Message, header::ContentType};

#[must_use]
pub struct EmailBuilder {
    from: Option<Mailbox>,
    to: Vec<Mailbox>,
    cc: Vec<Mailbox>,
    bcc: Vec<Mailbox>,
    reply_to: Option<Mailbox>,
    subject: Option<String>,
    body: Option<String>,
    is_html: bool,
}

impl EmailBuilder {
    pub fn new() -> Self {
        Self {
            from: None,
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: None,
            subject: None,
            body: None,
            is_html: false,
        }
    }

    pub fn from<T: AsRef<str>>(mut self, email: T) -> Result<Self, LettreError> {
        self.from = Some(email.as_ref().parse()?);
        Ok(self)
    }

    pub fn from_with_name<T: AsRef<str>, N: AsRef<str>>(
        mut self,
        name: N,
        email: T,
    ) -> Result<Self, LettreError> {
        self.from = Some(Mailbox::new(
            Some(name.as_ref().to_string()),
            email.as_ref().parse()?,
        ));
        Ok(self)
    }

    pub fn to<T: AsRef<str>>(mut self, email: T) -> Result<Self, LettreError> {
        self.to.push(email.as_ref().parse()?);
        Ok(self)
    }

    pub fn to_with_name<T: AsRef<str>, N: AsRef<str>>(
        mut self,
        name: N,
        email: T,
    ) -> Result<Self, LettreError> {
        self.to.push(Mailbox::new(
            Some(name.as_ref().to_string()),
            email.as_ref().parse()?,
        ));
        Ok(self)
    }

    pub fn cc<T: AsRef<str>>(mut self, email: T) -> Result<Self, LettreError> {
        self.cc.push(email.as_ref().parse()?);
        Ok(self)
    }

    pub fn bcc<T: AsRef<str>>(mut self, email: T) -> Result<Self, LettreError> {
        self.bcc.push(email.as_ref().parse()?);
        Ok(self)
    }

    pub fn reply_to<T: AsRef<str>>(mut self, email: T) -> Result<Self, LettreError> {
        self.reply_to = Some(email.as_ref().parse()?);
        Ok(self)
    }

    pub fn subject<T: AsRef<str>>(mut self, subject: T) -> Self {
        self.subject = Some(subject.as_ref().to_string());
        self
    }

    pub fn text_body<T: AsRef<str>>(mut self, body: T) -> Self {
        self.body = Some(body.as_ref().to_string());
        self.is_html = false;
        self
    }

    pub fn html_body<T: AsRef<str>>(mut self, body: T) -> Self {
        self.body = Some(body.as_ref().to_string());
        self.is_html = true;
        self
    }

    pub fn build(self) -> Result<Message, LettreError> {
        let from = self
            .from
            .ok_or_else(|| LettreError::MessageBuilder(lettre::error::Error::MissingFrom))?;

        if self.to.is_empty() {
            return Err(LettreError::MessageBuilder(lettre::error::Error::MissingTo));
        }

        let subject = self.subject.unwrap_or_default();
        let body = self.body.unwrap_or_default();

        let mut message_builder = Message::builder().from(from).subject(subject);

        for to in self.to {
            message_builder = message_builder.to(to);
        }

        for cc in self.cc {
            message_builder = message_builder.cc(cc);
        }

        for bcc in self.bcc {
            message_builder = message_builder.bcc(bcc);
        }

        if let Some(reply_to) = self.reply_to {
            message_builder = message_builder.reply_to(reply_to);
        }

        if self.is_html {
            message_builder = message_builder.header(ContentType::TEXT_HTML);
        } else {
            message_builder = message_builder.header(ContentType::TEXT_PLAIN);
        }

        Ok(message_builder.body(body)?)
    }
}

impl Default for EmailBuilder {
    fn default() -> Self {
        Self::new()
    }
}
