"""
Project Chatbot Service with RAG (Retrieval Augmented Generation)
Answers questions about the BotCore trading project using project documentation.
"""

import os
import re
import logging
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple
from datetime import datetime, timezone
import asyncio

logger = logging.getLogger(__name__)


# Project root directory - try multiple locations
def _find_project_root() -> Path:
    """Find the project root directory."""
    # Try relative to this file (local development)
    local_root = Path(__file__).parent.parent.parent
    if (local_root / "CLAUDE.md").exists():
        return local_root

    # Try Docker mounted volume paths
    docker_paths = [
        Path("/project"),  # Docker mounted project docs
        Path("/app"),  # If mounted at /app
        Path("/workspace"),  # If mounted at /workspace
        Path("/bot-core"),  # If mounted at /bot-core
    ]
    for docker_path in docker_paths:
        if (docker_path / "CLAUDE.md").exists():
            return docker_path

    # Try environment variable
    env_root = os.getenv("PROJECT_ROOT")
    if env_root and Path(env_root).exists():
        return Path(env_root)

    # Fallback: search upward from current working directory
    cwd = Path.cwd()
    for parent in [cwd] + list(cwd.parents):
        if (parent / "CLAUDE.md").exists():
            return parent

    # Last resort: return local_root anyway
    logger.warning(f"Could not find project root with CLAUDE.md, using: {local_root}")
    return local_root


PROJECT_ROOT = _find_project_root()
logger.info(f"📁 Project root: {PROJECT_ROOT}")


class DocumentIndex:
    """Simple in-memory document index for RAG."""

    def __init__(self):
        self.documents: List[Dict[str, Any]] = []
        self.indexed = False

    def add_document(self, path: str, content: str, doc_type: str, title: str = ""):
        """Add a document to the index."""
        # Split into chunks for better retrieval
        chunks = self._chunk_content(content, chunk_size=1500, overlap=200)

        for i, chunk in enumerate(chunks):
            self.documents.append(
                {
                    "path": path,
                    "content": chunk,
                    "doc_type": doc_type,
                    "title": title or Path(path).stem,
                    "chunk_index": i,
                    "total_chunks": len(chunks),
                }
            )

    def _chunk_content(
        self, content: str, chunk_size: int = 1500, overlap: int = 200
    ) -> List[str]:
        """Split content into overlapping chunks."""
        if len(content) <= chunk_size:
            return [content]

        chunks = []
        start = 0
        while start < len(content):
            end = start + chunk_size
            chunk = content[start:end]

            # Try to break at paragraph or sentence boundary
            if end < len(content):
                # Look for paragraph break
                last_para = chunk.rfind("\n\n")
                if last_para > chunk_size * 0.5:
                    chunk = chunk[:last_para]
                    end = start + last_para
                else:
                    # Look for sentence break
                    last_sentence = max(chunk.rfind(". "), chunk.rfind(".\n"))
                    if last_sentence > chunk_size * 0.5:
                        chunk = chunk[: last_sentence + 1]
                        end = start + last_sentence + 1

            chunks.append(chunk.strip())
            start = end - overlap

        return chunks

    def search(self, query: str, top_k: int = 5) -> List[Dict[str, Any]]:
        """Enhanced keyword-based search with Vietnamese support."""
        query_lower = query.lower()
        query_terms = set(query_lower.split())

        # Expand terms with synonyms and variations
        expanded_terms = set()
        synonym_map = {
            "bảo mật": ["security", "bảo mật", "bao mat", "secure"],
            "lớp": ["layer", "lớp", "lop", "layers", "tầng"],
            "rủi ro": ["risk", "rủi ro", "rui ro"],
            "chiến lược": ["strategy", "chiến lược", "chien luoc", "strategies"],
            "giao dịch": ["trading", "giao dịch", "giao dich", "trade"],
            "hoạt động": ["work", "hoạt động", "hoat dong", "working", "how"],
            "cấu hình": ["config", "cấu hình", "cau hinh", "configuration", "setup"],
            "paper": ["paper", "giả lập", "gia lap", "simulation"],
            "7": ["7", "bảy", "bay", "seven"],
        }

        for term in query_terms:
            expanded_terms.add(term)
            # Check if term matches any synonym key or value
            for key, synonyms in synonym_map.items():
                if term in key.lower() or any(term in s.lower() for s in synonyms):
                    expanded_terms.update(synonyms)
                    expanded_terms.add(key)

        # Extract numbers from query
        import re

        numbers = re.findall(r"\d+", query)
        expanded_terms.update(numbers)

        scored_docs = []
        for doc in self.documents:
            content_lower = doc["content"].lower()
            title_lower = doc["title"].lower()
            path_lower = doc["path"].lower()

            # Calculate relevance score
            score = 0

            # Exact phrase matching (highest priority)
            if query_lower in content_lower:
                score += 20
            if query_lower in title_lower:
                score += 30

            for term in expanded_terms:
                # Title match is worth more
                if term in title_lower:
                    score += 8
                # Path match (file name relevance)
                if term in path_lower:
                    score += 5
                # Content matches
                count = content_lower.count(term)
                score += min(count, 10)  # Cap at 10 per term

            # Boost certain doc types for certain queries
            if any(t in query_lower for t in ["api", "endpoint", "request"]):
                if "api" in doc["doc_type"].lower():
                    score += 5
            if any(
                t in query_lower
                for t in ["how", "làm sao", "cách", "hướng dẫn", "giải thích"]
            ):
                if "feature" in doc["doc_type"].lower() or "readme" in path_lower:
                    score += 5
            if any(
                t in query_lower
                for t in ["bảo mật", "security", "risk", "rủi ro", "lớp", "layer"]
            ):
                if (
                    "risk" in path_lower
                    or "security" in path_lower
                    or "layer" in path_lower
                ):
                    score += 10
                if "report" in doc["doc_type"].lower():
                    score += 5

            if score > 0:
                scored_docs.append((score, doc))

        # Sort by score and return top_k
        scored_docs.sort(key=lambda x: x[0], reverse=True)
        return [doc for _, doc in scored_docs[:top_k]]


class ProjectChatbot:
    """RAG-based chatbot for answering questions about the BotCore project."""

    def __init__(self, openai_client=None):
        self.openai_client = openai_client
        self.index = DocumentIndex()
        self.conversation_history: List[Dict[str, str]] = []
        self._indexed = False

    async def initialize(self):
        """Initialize the chatbot by indexing project documents."""
        if self._indexed:
            return

        logger.info("📚 Indexing project documentation...")
        await self._index_documents()
        self._indexed = True
        logger.info(f"✅ Indexed {len(self.index.documents)} document chunks")

    async def _index_documents(self):
        """Index all relevant project documents."""
        # Key documentation paths to index
        doc_paths = [
            # Main documentation
            ("docs/features", "feature"),
            ("docs/reports", "report"),
            ("docs/guides", "guide"),
            ("docs/plans", "plan"),
            ("specs/01-requirements/1.1-functional-requirements", "requirement"),
            ("specs/02-design/2.3-api", "api"),
            ("specs/02-design/2.5-components", "component"),
            ("docs", "guide"),
        ]

        # Also index CLAUDE.md for project overview
        claude_md = PROJECT_ROOT / "CLAUDE.md"
        if claude_md.exists():
            content = claude_md.read_text(encoding="utf-8")
            self.index.add_document(
                str(claude_md), content, "overview", "Project Overview (CLAUDE.md)"
            )

        # Index README files
        for readme in [
            "README.md",
            "nextjs-ui-dashboard/README.md",
            "rust-core-engine/README.md",
        ]:
            readme_path = PROJECT_ROOT / readme
            if readme_path.exists():
                content = readme_path.read_text(encoding="utf-8")
                self.index.add_document(str(readme_path), content, "readme", readme)

        # Index documentation directories
        for path_str, doc_type in doc_paths:
            dir_path = PROJECT_ROOT / path_str
            if not dir_path.exists():
                continue

            for md_file in dir_path.glob("**/*.md"):
                try:
                    content = md_file.read_text(encoding="utf-8")
                    relative_path = str(md_file.relative_to(PROJECT_ROOT))
                    self.index.add_document(
                        relative_path, content, doc_type, md_file.stem
                    )
                except Exception as e:
                    logger.warning(f"Failed to index {md_file}: {e}")

        self.index.indexed = True

    def _build_context(self, relevant_docs: List[Dict[str, Any]]) -> str:
        """Build context string from relevant documents."""
        if not relevant_docs:
            return "Không tìm thấy tài liệu liên quan."

        context_parts = []
        for doc in relevant_docs:
            header = f"📄 {doc['title']} ({doc['doc_type']})"
            context_parts.append(f"{header}\n{doc['content']}")

        return "\n\n---\n\n".join(context_parts)

    def _get_system_prompt(self) -> str:
        """Get system prompt for the chatbot."""
        return """Bạn là AI Assistant của BotCore - một nền tảng giao dịch cryptocurrency AI-powered.

THÔNG TIN VỀ BOTCORE:
- BotCore là hệ thống trading bot sử dụng AI (GPT-4, ML models) để phân tích và giao dịch crypto futures
- 3 services chính: Rust Core Engine (trading), Python AI Service (ML/AI), Next.js Dashboard (UI)
- Hỗ trợ 4 chiến lược: RSI, MACD, Bollinger Bands, Volume Analysis
- Paper Trading mode để test trước khi trade thật
- Real-time WebSocket để cập nhật giá và signals

HƯỚNG DẪN TRẢ LỜI:
1. Trả lời bằng tiếng Việt, ngắn gọn và dễ hiểu
2. Sử dụng thông tin từ CONTEXT được cung cấp
3. Nếu không tìm thấy trong context, nói rõ và đưa ra câu trả lời chung
4. Format với bullet points và emoji khi phù hợp
5. Nếu là câu hỏi kỹ thuật, đưa ra code example hoặc config example
6. Luôn cảnh báo về rủi ro khi nói về trading thật

KHÔNG ĐƯỢC:
- Đưa ra lời khuyên tài chính cụ thể
- Hứa hẹn về lợi nhuận
- Khuyến khích trade với số tiền lớn"""

    async def chat(self, message: str, include_history: bool = True) -> Dict[str, Any]:
        """Process a chat message and return response."""
        if not self._indexed:
            await self.initialize()

        # Search for relevant documents
        relevant_docs = self.index.search(message, top_k=5)
        context = self._build_context(relevant_docs)

        # Build messages for GPT
        messages = [{"role": "system", "content": self._get_system_prompt()}]

        # Add conversation history (last 6 messages)
        if include_history and self.conversation_history:
            messages.extend(self.conversation_history[-6:])

        # Add context and user message
        user_content = f"""CONTEXT TỪ TÀI LIỆU DỰ ÁN:
{context}

---

CÂU HỎI CỦA USER:
{message}"""

        messages.append({"role": "user", "content": user_content})

        # Call GPT if available
        if self.openai_client:
            try:
                response = await self.openai_client.chat_completions_create(
                    model="gpt-5-mini",
                    messages=messages,
                    max_tokens=1000,
                    temperature=0.7,
                )

                assistant_message = response["choices"][0]["message"]["content"]

                # Update conversation history
                self.conversation_history.append({"role": "user", "content": message})
                self.conversation_history.append(
                    {"role": "assistant", "content": assistant_message}
                )

                # Keep history limited
                if len(self.conversation_history) > 20:
                    self.conversation_history = self.conversation_history[-20:]

                return {
                    "success": True,
                    "message": assistant_message,
                    "sources": [
                        {"title": doc["title"], "path": doc["path"]}
                        for doc in relevant_docs[:3]
                    ],
                    "confidence": 0.9 if relevant_docs else 0.6,
                    "type": "rag",
                    "tokens_used": response.get("usage", {}),
                }

            except Exception as e:
                logger.error(f"GPT-4 error: {e}")
                return await self._fallback_response(message, relevant_docs)
        else:
            return await self._fallback_response(message, relevant_docs)

    async def _fallback_response(
        self, message: str, relevant_docs: List[Dict[str, Any]]
    ) -> Dict[str, Any]:
        """Fallback response when GPT is not available."""
        # Simple keyword-based responses
        message_lower = message.lower()

        response = ""

        if any(word in message_lower for word in ["hoạt động", "làm việc", "work"]):
            response = """🤖 **BotCore hoạt động như thế nào:**

1. **Thu thập dữ liệu**: Kết nối Binance API để lấy giá real-time
2. **Phân tích AI**:
   - GPT-4 phân tích sentiment và xu hướng
   - ML models (LSTM, GRU) dự đoán giá
3. **Tạo signals**: Kết hợp RSI, MACD, Bollinger Bands, Volume
4. **Thực thi lệnh**: Auto trade hoặc thông báo để bạn quyết định
5. **Quản lý rủi ro**: Stop-loss, take-profit, position sizing

💡 Khuyên dùng Paper Trading để test trước!"""

        elif any(
            word in message_lower for word in ["bắt đầu", "start", "setup", "cài đặt"]
        ):
            response = """🚀 **Bắt đầu với BotCore:**

1. **Clone repo và setup:**
   ```bash
   git clone <repo>
   cp .env.example .env
   ./scripts/generate-secrets.sh
   ```

2. **Cấu hình API keys:**
   - Tạo Binance API key (chỉ cần quyền trade)
   - Thêm OpenAI API key cho AI features

3. **Chạy services:**
   ```bash
   ./scripts/bot.sh start --memory-optimized
   ```

4. **Truy cập Dashboard:** http://localhost:3000

⚠️ Bắt đầu với Paper Trading mode để làm quen!"""

        elif any(
            word in message_lower for word in ["chiến lược", "strategy", "rsi", "macd"]
        ):
            response = """📊 **Các chiến lược trading của BotCore:**

1. **RSI Strategy** (Win rate: 62%)
   - Mua khi RSI < 30, bán khi RSI > 70
   - Timeframe: 15m, 1h

2. **MACD Strategy** (Win rate: 58%)
   - Theo dõi MACD crossover
   - Kết hợp histogram divergence

3. **Bollinger Bands** (Win rate: 60%)
   - Trade breakout và mean reversion
   - Kết hợp với volume

4. **Volume Strategy** (Win rate: 52%)
   - Phát hiện volume spike
   - Xác nhận trend strength

💡 Có thể bật/tắt từng strategy trong Settings"""

        elif any(
            word in message_lower for word in ["an toàn", "bảo mật", "security", "risk"]
        ):
            response = """🔒 **Bảo mật & Quản lý rủi ro:**

**Bảo mật:**
- API keys mã hóa AES-256
- Tiền luôn trong tài khoản Binance của bạn
- Bot không có quyền rút tiền
- JWT authentication cho dashboard

**Quản lý rủi ro:**
- Stop-loss tự động (1-15%)
- Take-profit để chốt lời
- Daily loss limit (5% max)
- Cool-down sau 5 losses liên tiếp
- Position sizing theo % balance

⚠️ **Quan trọng:** Chỉ trade với số tiền bạn có thể mất!"""

        else:
            # Return relevant docs content as fallback
            if relevant_docs:
                response = f"""📚 Dựa trên tài liệu dự án:

{relevant_docs[0]['content'][:800]}...

💡 Bạn có thể hỏi cụ thể hơn để tôi trả lời chính xác hơn!"""
            else:
                response = """🤔 Tôi chưa tìm thấy thông tin cụ thể về câu hỏi này trong tài liệu.

Bạn có thể hỏi về:
• Bot hoạt động như thế nào?
• Cách bắt đầu sử dụng?
• Các chiến lược trading?
• Bảo mật và quản lý rủi ro?
• Cấu hình API keys?

Hoặc liên hệ support để được hỗ trợ trực tiếp!"""

        return {
            "success": True,
            "message": response,
            "sources": [
                {"title": doc["title"], "path": doc["path"]}
                for doc in relevant_docs[:3]
            ],
            "confidence": 0.7 if relevant_docs else 0.5,
            "type": "fallback",
        }

    def clear_history(self):
        """Clear conversation history."""
        self.conversation_history = []

    def get_suggested_questions(self) -> List[str]:
        """Get suggested questions for users."""
        return [
            "Bot hoạt động như thế nào?",
            "Làm sao để bắt đầu sử dụng?",
            "Các chiến lược trading là gì?",
            "Bot có an toàn không?",
            "Vốn tối thiểu là bao nhiêu?",
            "Cách cấu hình API keys?",
            "Paper trading là gì?",
            "Cách xem kết quả trading?",
        ]


# Singleton instance
_chatbot_instance: Optional[ProjectChatbot] = None


async def get_chatbot(openai_client=None) -> ProjectChatbot:
    """Get or create chatbot instance."""
    global _chatbot_instance

    if _chatbot_instance is None:
        _chatbot_instance = ProjectChatbot(openai_client)
        await _chatbot_instance.initialize()
    elif openai_client and _chatbot_instance.openai_client is None:
        _chatbot_instance.openai_client = openai_client

    return _chatbot_instance
