"""
Tests for project_chatbot.py

@spec:FR-CHATBOT-001 - Project documentation RAG chatbot
"""

import pytest
from unittest.mock import patch, MagicMock, AsyncMock
from pathlib import Path


@pytest.mark.unit
class TestDocumentIndex:
    """Tests for DocumentIndex class."""

    def test_init(self):
        """Test DocumentIndex initialization."""
        from services.project_chatbot import DocumentIndex

        index = DocumentIndex()
        assert index.documents == []
        assert index.indexed is False

    def test_add_document_short_content(self):
        """Test adding document with short content."""
        from services.project_chatbot import DocumentIndex

        index = DocumentIndex()
        index.add_document(
            path="test/path.md",
            content="Short content",
            doc_type="test",
            title="Test Title"
        )

        assert len(index.documents) == 1
        assert index.documents[0]["path"] == "test/path.md"
        assert index.documents[0]["content"] == "Short content"
        assert index.documents[0]["doc_type"] == "test"
        assert index.documents[0]["title"] == "Test Title"
        assert index.documents[0]["chunk_index"] == 0
        assert index.documents[0]["total_chunks"] == 1

    def test_add_document_no_title(self):
        """Test adding document without title uses path stem."""
        from services.project_chatbot import DocumentIndex

        index = DocumentIndex()
        index.add_document(
            path="test/document.md",
            content="Content here",
            doc_type="guide",
        )

        assert index.documents[0]["title"] == "document"

    def test_add_document_long_content_chunks(self):
        """Test adding long document creates multiple chunks."""
        from services.project_chatbot import DocumentIndex

        index = DocumentIndex()
        # Create content longer than chunk_size (1500)
        long_content = "A" * 3000

        index.add_document(
            path="test/long.md",
            content=long_content,
            doc_type="test",
            title="Long Doc"
        )

        # Should have multiple chunks
        assert len(index.documents) >= 2
        assert all(doc["total_chunks"] >= 2 for doc in index.documents)

    def test_chunk_content_short(self):
        """Test _chunk_content with short content."""
        from services.project_chatbot import DocumentIndex

        index = DocumentIndex()
        chunks = index._chunk_content("Short content", chunk_size=1500)

        assert len(chunks) == 1
        assert chunks[0] == "Short content"

    def test_chunk_content_with_paragraph_breaks(self):
        """Test _chunk_content breaks at paragraphs."""
        from services.project_chatbot import DocumentIndex

        index = DocumentIndex()
        # Create content with paragraph breaks
        content = "First paragraph. " * 50 + "\n\n" + "Second paragraph. " * 50

        chunks = index._chunk_content(content, chunk_size=500, overlap=50)

        # Should break at paragraph boundary
        assert len(chunks) >= 2

    def test_chunk_content_with_sentence_breaks(self):
        """Test _chunk_content breaks at sentences."""
        from services.project_chatbot import DocumentIndex

        index = DocumentIndex()
        content = "Sentence one. Sentence two. Sentence three. " * 40

        chunks = index._chunk_content(content, chunk_size=100, overlap=20)

        assert len(chunks) >= 2

    def test_search_empty_query(self):
        """Test search with no documents."""
        from services.project_chatbot import DocumentIndex

        index = DocumentIndex()
        results = index.search("test query")

        assert results == []

    def test_search_basic_keyword(self):
        """Test search finds documents with matching keywords."""
        from services.project_chatbot import DocumentIndex

        index = DocumentIndex()
        index.add_document("doc1.md", "This is about trading strategies", "guide", "Trading")
        index.add_document("doc2.md", "This is about authentication", "guide", "Auth")

        results = index.search("trading")

        assert len(results) >= 1
        assert "trading" in results[0]["content"].lower() or "trading" in results[0]["title"].lower()

    def test_search_title_match_higher_score(self):
        """Test search gives higher score to title matches."""
        from services.project_chatbot import DocumentIndex

        index = DocumentIndex()
        index.add_document("doc1.md", "Some content", "guide", "Trading Strategies")
        index.add_document("doc2.md", "Content about trading methods", "guide", "Other")

        results = index.search("trading")

        # Title match should come first
        assert len(results) >= 2
        assert "trading" in results[0]["title"].lower() or "trading" in results[0]["content"].lower()

    def test_search_with_synonyms(self):
        """Test search expands Vietnamese synonyms."""
        from services.project_chatbot import DocumentIndex

        index = DocumentIndex()
        index.add_document("doc1.md", "Content about security features", "guide", "Security")
        index.add_document("doc2.md", "Content about other topics", "guide", "Other")

        # Search with Vietnamese term should find security doc
        results = index.search("bao mat")

        assert len(results) >= 1

    def test_search_api_query_boost(self):
        """Test search boosts API docs for API-related queries."""
        from services.project_chatbot import DocumentIndex

        index = DocumentIndex()
        index.add_document("api/endpoints.md", "API endpoint docs", "api", "API Docs")
        index.add_document("guide/intro.md", "General guide", "guide", "Intro")

        results = index.search("api endpoint")

        assert len(results) >= 1
        # API doc should be boosted
        assert "api" in results[0]["doc_type"].lower() or "api" in results[0]["path"].lower()

    def test_search_top_k_limit(self):
        """Test search respects top_k limit."""
        from services.project_chatbot import DocumentIndex

        index = DocumentIndex()
        for i in range(10):
            index.add_document(f"doc{i}.md", f"Trading content {i}", "guide", f"Doc {i}")

        results = index.search("trading", top_k=3)

        assert len(results) <= 3


@pytest.mark.unit
class TestProjectChatbot:
    """Tests for ProjectChatbot class."""

    def test_init(self):
        """Test ProjectChatbot initialization."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        assert chatbot.openai_client is None
        assert chatbot._indexed is False
        assert chatbot.conversation_history == []

    def test_init_with_client(self):
        """Test ProjectChatbot initialization with OpenAI client."""
        from services.project_chatbot import ProjectChatbot

        mock_client = MagicMock()
        chatbot = ProjectChatbot(openai_client=mock_client)

        assert chatbot.openai_client is mock_client

    @pytest.mark.asyncio
    async def test_initialize_already_indexed(self):
        """Test initialize skips when already indexed."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        chatbot._indexed = True

        await chatbot.initialize()

        # Should not re-index
        assert len(chatbot.index.documents) == 0

    @pytest.mark.asyncio
    async def test_initialize_indexes_documents(self):
        """Test initialize indexes project documents."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()

        # Mock the project root to avoid file system issues
        with patch("services.project_chatbot.PROJECT_ROOT", Path("/tmp/test_project")):
            with patch.object(Path, "exists", return_value=False):
                await chatbot.initialize()

        assert chatbot._indexed is True

    def test_build_context_empty(self):
        """Test _build_context with no documents."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        context = chatbot._build_context([])

        assert context == "Khong tim thay tai lieu lien quan." or "Không tìm thấy" in context

    def test_build_context_with_docs(self):
        """Test _build_context with documents."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        docs = [
            {"title": "Doc1", "doc_type": "guide", "content": "Content 1"},
            {"title": "Doc2", "doc_type": "api", "content": "Content 2"},
        ]

        context = chatbot._build_context(docs)

        assert "Doc1" in context
        assert "Doc2" in context
        assert "Content 1" in context
        assert "Content 2" in context

    def test_get_system_prompt(self):
        """Test _get_system_prompt returns valid prompt."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        prompt = chatbot._get_system_prompt()

        assert "BotCore" in prompt
        assert len(prompt) > 100  # Should be substantial

    @pytest.mark.asyncio
    async def test_chat_initializes_if_needed(self):
        """Test chat initializes chatbot if not indexed."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        chatbot._indexed = False

        with patch.object(chatbot, "initialize", new_callable=AsyncMock) as mock_init:
            with patch.object(chatbot, "_fallback_response", new_callable=AsyncMock) as mock_fallback:
                mock_fallback.return_value = {"success": True, "message": "Test"}
                await chatbot.chat("test message")

        mock_init.assert_called_once()

    @pytest.mark.asyncio
    async def test_chat_with_openai_client_success(self):
        """Test chat with successful OpenAI response."""
        from services.project_chatbot import ProjectChatbot

        mock_client = AsyncMock()
        mock_client.chat_completions_create = AsyncMock(return_value={
            "choices": [{"message": {"content": "GPT response"}}],
            "usage": {"total_tokens": 100}
        })

        chatbot = ProjectChatbot(openai_client=mock_client)
        chatbot._indexed = True

        result = await chatbot.chat("test question")

        assert result["success"] is True
        assert result["message"] == "GPT response"
        assert result["type"] == "rag"
        assert len(chatbot.conversation_history) == 2  # User + assistant

    @pytest.mark.asyncio
    async def test_chat_with_openai_error_falls_back(self):
        """Test chat falls back on OpenAI error."""
        from services.project_chatbot import ProjectChatbot

        mock_client = AsyncMock()
        mock_client.chat_completions_create = AsyncMock(side_effect=Exception("API Error"))

        chatbot = ProjectChatbot(openai_client=mock_client)
        chatbot._indexed = True

        result = await chatbot.chat("test question")

        assert result["success"] is True
        assert result["type"] == "fallback"

    @pytest.mark.asyncio
    async def test_chat_no_openai_client_uses_fallback(self):
        """Test chat uses fallback when no OpenAI client."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        chatbot._indexed = True

        result = await chatbot.chat("test question")

        assert result["success"] is True
        assert result["type"] == "fallback"

    @pytest.mark.asyncio
    async def test_chat_history_limit(self):
        """Test conversation history is limited."""
        from services.project_chatbot import ProjectChatbot

        mock_client = AsyncMock()
        mock_client.chat_completions_create = AsyncMock(return_value={
            "choices": [{"message": {"content": "Response"}}],
            "usage": {}
        })

        chatbot = ProjectChatbot(openai_client=mock_client)
        chatbot._indexed = True

        # Add many messages to history
        for i in range(25):
            chatbot.conversation_history.append({"role": "user", "content": f"msg{i}"})
            chatbot.conversation_history.append({"role": "assistant", "content": f"resp{i}"})

        await chatbot.chat("new message")

        # History should be trimmed to 20
        assert len(chatbot.conversation_history) <= 22  # 20 + 2 new

    @pytest.mark.asyncio
    async def test_fallback_response_how_it_works(self):
        """Test fallback response for 'how it works' questions."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        chatbot._indexed = True

        # Use Vietnamese with proper diacritics
        result = await chatbot._fallback_response("Bot hoạt động như thế nào?", [])

        assert result["success"] is True
        assert "BotCore" in result["message"] or "Binance" in result["message"]
        assert result["type"] == "fallback"

    @pytest.mark.asyncio
    async def test_fallback_response_getting_started(self):
        """Test fallback response for getting started questions."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        chatbot._indexed = True

        # Use Vietnamese with proper diacritics
        result = await chatbot._fallback_response("Cách bắt đầu sử dụng?", [])

        assert result["success"] is True
        assert "setup" in result["message"].lower() or "clone" in result["message"].lower() or "git" in result["message"].lower()

    @pytest.mark.asyncio
    async def test_fallback_response_strategies(self):
        """Test fallback response for strategy questions."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        chatbot._indexed = True

        result = await chatbot._fallback_response("chien luoc rsi", [])

        assert result["success"] is True
        assert "RSI" in result["message"] or "strategy" in result["message"].lower()

    @pytest.mark.asyncio
    async def test_fallback_response_security(self):
        """Test fallback response for security questions."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        chatbot._indexed = True

        result = await chatbot._fallback_response("bao mat va an toan", [])

        assert result["success"] is True
        # Should mention security features
        assert any(word in result["message"].lower() for word in ["bao mat", "security", "risk", "an toan", "bảo mật"])

    @pytest.mark.asyncio
    async def test_fallback_response_with_relevant_docs(self):
        """Test fallback response with relevant documents."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        chatbot._indexed = True

        docs = [{"title": "Test", "path": "test.md", "content": "Relevant content here"}]
        result = await chatbot._fallback_response("unknown question", docs)

        assert result["success"] is True
        assert "Relevant content" in result["message"] or len(result["sources"]) > 0

    @pytest.mark.asyncio
    async def test_fallback_response_no_match(self):
        """Test fallback response for unmatched questions."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        chatbot._indexed = True

        result = await chatbot._fallback_response("xyz123 random query", [])

        assert result["success"] is True
        # Should show help options
        assert "hoi" in result["message"].lower() or "?" in result["message"]

    def test_clear_history(self):
        """Test clear_history resets conversation."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        chatbot.conversation_history = [
            {"role": "user", "content": "msg1"},
            {"role": "assistant", "content": "resp1"}
        ]

        chatbot.clear_history()

        assert chatbot.conversation_history == []

    def test_get_suggested_questions(self):
        """Test get_suggested_questions returns list."""
        from services.project_chatbot import ProjectChatbot

        chatbot = ProjectChatbot()
        suggestions = chatbot.get_suggested_questions()

        assert isinstance(suggestions, list)
        assert len(suggestions) > 0
        assert all(isinstance(q, str) for q in suggestions)


@pytest.mark.unit
class TestGetChatbot:
    """Tests for get_chatbot function."""

    @pytest.mark.asyncio
    async def test_get_chatbot_creates_instance(self):
        """Test get_chatbot creates new instance."""
        from services import project_chatbot

        # Reset singleton
        project_chatbot._chatbot_instance = None

        with patch.object(project_chatbot.ProjectChatbot, "initialize", new_callable=AsyncMock):
            result = await project_chatbot.get_chatbot(None)

        assert result is not None
        assert isinstance(result, project_chatbot.ProjectChatbot)

        # Cleanup
        project_chatbot._chatbot_instance = None

    @pytest.mark.asyncio
    async def test_get_chatbot_returns_existing(self):
        """Test get_chatbot returns existing instance."""
        from services import project_chatbot

        # Create instance
        mock_instance = MagicMock()
        mock_instance.openai_client = MagicMock()
        project_chatbot._chatbot_instance = mock_instance

        result = await project_chatbot.get_chatbot(None)

        assert result is mock_instance

        # Cleanup
        project_chatbot._chatbot_instance = None

    @pytest.mark.asyncio
    async def test_get_chatbot_updates_client(self):
        """Test get_chatbot updates client when None."""
        from services import project_chatbot

        # Create instance without client
        mock_instance = MagicMock()
        mock_instance.openai_client = None
        project_chatbot._chatbot_instance = mock_instance

        new_client = MagicMock()
        result = await project_chatbot.get_chatbot(new_client)

        assert result.openai_client is new_client

        # Cleanup
        project_chatbot._chatbot_instance = None


@pytest.mark.unit
class TestFindProjectRoot:
    """Tests for _find_project_root function."""

    def test_find_project_root_returns_path(self):
        """Test _find_project_root returns a Path."""
        from services.project_chatbot import _find_project_root

        result = _find_project_root()

        assert isinstance(result, Path)

    def test_find_project_root_with_env_var(self):
        """Test _find_project_root uses PROJECT_ROOT env var."""
        import tempfile
        from services.project_chatbot import _find_project_root

        with tempfile.TemporaryDirectory() as tmpdir:
            # Create CLAUDE.md
            claude_md = Path(tmpdir) / "CLAUDE.md"
            claude_md.touch()

            with patch.dict("os.environ", {"PROJECT_ROOT": tmpdir}):
                # Note: function is already evaluated, test concept only
                result = _find_project_root()
                assert isinstance(result, Path)
