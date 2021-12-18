(function() {var implementors = {};
implementors["ipp"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://docs.rs/http/0.2.5/http/error/struct.Error.html\" title=\"struct http::error::Error\">Error</a>&gt; for <a class=\"enum\" href=\"ipp/client/enum.IppError.html\" title=\"enum ipp::client::IppError\">IppError</a>","synthetic":false,"types":["ipp::client::IppError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://docs.rs/reqwest/0.11.7/reqwest/error/struct.Error.html\" title=\"struct reqwest::error::Error\">Error</a>&gt; for <a class=\"enum\" href=\"ipp/client/enum.IppError.html\" title=\"enum ipp::client::IppError\">IppError</a>","synthetic":false,"types":["ipp::client::IppError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.57.0/std/io/error/struct.Error.html\" title=\"struct std::io::error::Error\">Error</a>&gt; for <a class=\"enum\" href=\"ipp/client/enum.IppError.html\" title=\"enum ipp::client::IppError\">IppError</a>","synthetic":false,"types":["ipp::client::IppError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"ipp/parser/enum.IppParseError.html\" title=\"enum ipp::parser::IppParseError\">IppParseError</a>&gt; for <a class=\"enum\" href=\"ipp/client/enum.IppError.html\" title=\"enum ipp::client::IppError\">IppError</a>","synthetic":false,"types":["ipp::client::IppError"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://docs.rs/http/0.2.5/http/uri/struct.InvalidUri.html\" title=\"struct http::uri::InvalidUri\">InvalidUri</a>&gt; for <a class=\"enum\" href=\"ipp/client/enum.IppError.html\" title=\"enum ipp::client::IppError\">IppError</a>","synthetic":false,"types":["ipp::client::IppError"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"ipp/operation/trait.IppOperation.html\" title=\"trait ipp::operation::IppOperation\">IppOperation</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;T&gt; for <a class=\"struct\" href=\"ipp/request/struct.IppRequestResponse.html\" title=\"struct ipp::request::IppRequestResponse\">IppRequestResponse</a>","synthetic":false,"types":["ipp::request::IppRequestResponse"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.57.0/std/io/error/struct.Error.html\" title=\"struct std::io::error::Error\">Error</a>&gt; for <a class=\"enum\" href=\"ipp/parser/enum.IppParseError.html\" title=\"enum ipp::parser::IppParseError\">IppParseError</a>","synthetic":false,"types":["ipp::parser::IppParseError"]},{"text":"impl&lt;R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;R&gt; for <a class=\"struct\" href=\"ipp/reader/struct.AsyncIppReader.html\" title=\"struct ipp::reader::AsyncIppReader\">AsyncIppReader</a>&lt;R&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;R: 'static + AsyncRead + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":false,"types":["ipp::reader::AsyncIppReader"]},{"text":"impl&lt;R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;R&gt; for <a class=\"struct\" href=\"ipp/reader/struct.IppReader.html\" title=\"struct ipp::reader::IppReader\">IppReader</a>&lt;R&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;R: 'static + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/std/io/trait.Read.html\" title=\"trait std::io::Read\">Read</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.57.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":false,"types":["ipp::reader::IppReader"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()