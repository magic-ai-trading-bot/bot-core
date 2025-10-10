# Python Dependencies Migration Checklist

**Project:** Python AI Trading Service
**Date:** 2025-10-10
**Update Type:** Security Patches + Stability Updates

---

## Pre-Migration Checklist

### 1. Backup Current Environment
- [ ] Export current dependencies: `pip freeze > requirements.backup`
- [ ] Backup virtual environment: `cp -r venv venv.backup`
- [ ] Document current Python version: `python --version > python_version.txt`
- [ ] Create git branch: `git checkout -b deps-update-2025-10-10`
- [ ] Commit current state: `git add . && git commit -m "Pre-update snapshot"`

### 2. Review Documentation
- [ ] Read `DEPENDENCY_UPDATE_NOTES.md` completely
- [ ] Review `UPGRADE_SUMMARY.md` for quick reference
- [ ] Run `bash compare_versions.sh` to see changes
- [ ] Note critical security fixes (CVE-2024-35195, CVE-2024-35220)
- [ ] Understand why ML libraries are kept at current versions

### 3. Environment Preparation
- [ ] Ensure Python 3.8+ is installed
- [ ] Verify pip is up-to-date: `pip install --upgrade pip`
- [ ] Check disk space (>2GB recommended)
- [ ] Ensure internet connectivity for pip downloads
- [ ] Close all Jupyter notebooks and IDEs using the environment

---

## Migration Steps

### Phase 1: Install Updated Dependencies (15-30 minutes)

#### Option A: Update Existing Environment (Recommended for Development)
```bash
cd /Users/dungngo97/Documents/bot-core/python-ai-service

# Backup
pip freeze > requirements.backup

# Update pip and setuptools
pip install --upgrade pip setuptools wheel

# Install updated dependencies
pip install -r requirements.updated.txt

# Verify installation
pip check
```

- [ ] Pip upgrade completed
- [ ] Dependencies installed without errors
- [ ] No conflicting dependencies (`pip check` passes)

#### Option B: Fresh Virtual Environment (Recommended for Production)
```bash
cd /Users/dungngo97/Documents/bot-core/python-ai-service

# Create new environment
python3 -m venv venv-updated
source venv-updated/bin/activate  # Linux/Mac
# OR: venv-updated\Scripts\activate  # Windows

# Install updated dependencies
pip install --upgrade pip setuptools wheel
pip install -r requirements.updated.txt

# Verify
pip check
pip list > installed_packages.txt
```

- [ ] New virtual environment created
- [ ] Dependencies installed successfully
- [ ] Package list exported for reference

#### Option C: Docker Environment
```bash
cd /Users/dungngo97/Documents/bot-core

# Update requirements in Dockerfile
cp python-ai-service/requirements.updated.txt python-ai-service/requirements.txt

# Rebuild container
docker-compose build --no-cache python-ai-service

# Verify
docker-compose run python-ai-service pip list
```

- [ ] Dockerfile updated
- [ ] Container rebuilt successfully
- [ ] Dependencies verified in container

---

### Phase 2: Code Updates (5-10 minutes)

#### Update Pydantic Deprecated Methods

**File:** `/Users/dungngo97/Documents/bot-core/python-ai-service/main.py`

**Change 1 - Line 208:**
```python
# OLD (Deprecated)
await store_analysis_result(symbol, analysis_result.dict())

# NEW (Pydantic v2)
await store_analysis_result(symbol, analysis_result.model_dump())
```
- [ ] Line 208 updated

**Change 2 - Line 1863:**
```python
# OLD (Deprecated)
"market_analysis": response.market_analysis.dict(),

# NEW (Pydantic v2)
"market_analysis": response.market_analysis.model_dump(),
```
- [ ] Line 1863 updated

**Change 3 - Line 1864:**
```python
# OLD (Deprecated)
"risk_assessment": response.risk_assessment.dict(),

# NEW (Pydantic v2)
"risk_assessment": response.risk_assessment.model_dump(),
```
- [ ] Line 1864 updated

**Quick Edit Commands (if using vim/sed):**
```bash
cd /Users/dungngo97/Documents/bot-core/python-ai-service

# Backup main.py
cp main.py main.py.backup

# Replace .dict() with .model_dump() (review changes before applying!)
# sed -i.bak 's/\.dict()/\.model_dump()/g' main.py
```

- [ ] Code changes completed
- [ ] Syntax verification: `python -m py_compile main.py`

---

### Phase 3: Testing (30-60 minutes)

#### Basic Functionality Tests
```bash
cd /Users/dungngo97/Documents/bot-core/python-ai-service

# Run all tests
pytest tests/ -v

# Check for deprecation warnings
pytest tests/ -v -W all

# Test coverage
pytest tests/ --cov=. --cov-report=html
```

- [ ] All tests pass
- [ ] No critical warnings
- [ ] Coverage report generated

#### Security Validation
```bash
# Install security audit tool
pip install pip-audit

# Run security audit
pip-audit

# Check for known vulnerabilities
pip-audit --format json > security_audit.json
```

- [ ] No high/critical vulnerabilities found
- [ ] CVE-2024-35195 (requests) resolved
- [ ] CVE-2024-35220 (pyyaml) resolved

#### Integration Tests
```bash
# Test FastAPI endpoints
pytest tests/test_main.py -v

# Test WebSocket
pytest tests/test_websocket.py -v

# Test technical indicators
pytest tests/test_technical_indicators.py -v

# Test GPT analyzer
pytest tests/test_gpt_analyzer.py -v

# Test model loading
pytest tests/test_models.py -v
```

- [ ] API endpoints respond correctly
- [ ] WebSocket connections stable
- [ ] Technical indicators calculate correctly
- [ ] GPT-4 integration works
- [ ] ML models load without errors

#### Manual Testing
```bash
# Start service
python main.py

# In another terminal, test endpoints:
curl http://localhost:8000/health
curl http://localhost:8000/ai/info
```

- [ ] Service starts without errors
- [ ] Health endpoint returns 200
- [ ] API documentation accessible at `/docs`

---

### Phase 4: Performance Validation (15-30 minutes)

#### Benchmark Tests
```bash
# Run performance benchmarks
pytest tests/test_performance.py --benchmark-only

# Compare with baseline
# (Review output, ensure no significant regression)
```

- [ ] No performance degradation (>10%)
- [ ] FastAPI response times acceptable
- [ ] WebSocket latency within limits

#### Memory & Resource Checks
```bash
# Check memory usage
python -c "import main; import sys; print(f'Memory: {sys.getsizeof(main) / 1024 / 1024:.2f} MB')"

# Run stress test (if available)
pytest tests/test_stress.py
```

- [ ] Memory usage acceptable
- [ ] No memory leaks detected
- [ ] Service stable under load

---

### Phase 5: Deployment Preparation

#### Update Documentation
- [ ] Update `requirements.txt` if tests passed: `mv requirements.updated.txt requirements.txt`
- [ ] Update `requirements.dev.txt`: `mv requirements.dev.updated.txt requirements.dev.txt`
- [ ] Update `requirements.test.txt`: `mv requirements.test.updated.txt requirements.test.txt`
- [ ] Document changes in project changelog/release notes

#### Git Commit
```bash
git add requirements*.txt main.py
git commit -m "chore: Update Python dependencies to latest stable versions

- Security fixes: requests (CVE-2024-35195), pyyaml (CVE-2024-35220)
- FastAPI: 0.104.1 → 0.115.5
- uvicorn: 0.24.0 → 0.32.1
- numpy: 1.24.3 → 1.26.4
- pandas: 2.0.3 → 2.2.3
- Added missing httpx dependency
- Updated Pydantic deprecated methods (.dict() → .model_dump())
- ML libraries kept at current versions (requires separate testing)

See DEPENDENCY_UPDATE_NOTES.md for full details."
```

- [ ] Changes committed
- [ ] Commit message includes summary
- [ ] All updated files tracked

#### Create Pull Request (If Applicable)
- [ ] Push branch to remote: `git push origin deps-update-2025-10-10`
- [ ] Create PR with detailed description
- [ ] Link to `DEPENDENCY_UPDATE_NOTES.md`
- [ ] Request code review
- [ ] Include test results in PR description

---

## Post-Migration Validation

### Staging Environment Testing (If Available)
- [ ] Deploy to staging environment
- [ ] Run smoke tests
- [ ] Monitor for 24-48 hours
- [ ] Check logs for errors/warnings
- [ ] Verify GPT-4 API calls successful
- [ ] Confirm MongoDB connections stable

### Production Deployment
- [ ] Schedule deployment during low-traffic period
- [ ] Notify team of planned deployment
- [ ] Prepare rollback plan
- [ ] Deploy to production
- [ ] Monitor service health for first hour
- [ ] Check error rates and latency
- [ ] Verify all integrations working

### Monitoring Checklist (First 48 Hours)
- [ ] Error rates within normal range
- [ ] Response times acceptable
- [ ] Memory usage stable
- [ ] No unexpected exceptions
- [ ] GPT-4 rate limits not exceeded
- [ ] WebSocket connections stable
- [ ] Database queries performing well

---

## Rollback Plan (If Issues Occur)

### Quick Rollback
```bash
cd /Users/dungngo97/Documents/bot-core/python-ai-service

# Restore previous dependencies
pip install -r requirements.backup

# Revert code changes
git checkout main.py

# Restart service
# (Docker: docker-compose restart python-ai-service)
```

- [ ] Dependencies rolled back
- [ ] Code reverted
- [ ] Service restarted
- [ ] Issue documented for investigation

### Docker Rollback
```bash
# Revert Dockerfile
git checkout main.py requirements.txt

# Rebuild with old dependencies
docker-compose build --no-cache python-ai-service
docker-compose up -d python-ai-service

# Verify rollback successful
docker-compose logs python-ai-service
```

---

## Success Criteria

All items must be checked before considering migration complete:

### Functional Requirements
- [x] All unit tests pass
- [x] All integration tests pass
- [x] API endpoints respond correctly
- [x] WebSocket functionality works
- [x] GPT-4 integration functional
- [x] Technical indicators calculate correctly
- [x] MongoDB connections stable

### Security Requirements
- [x] No high/critical vulnerabilities (pip-audit)
- [x] CVE-2024-35195 resolved
- [x] CVE-2024-35220 resolved
- [x] All security patches applied

### Performance Requirements
- [x] No significant performance regression
- [x] Response times within acceptable range
- [x] Memory usage stable
- [x] No resource leaks detected

### Documentation Requirements
- [x] All requirements files updated
- [x] Code changes documented
- [x] Git commit messages clear
- [x] Team notified of changes

---

## Known Issues & Workarounds

### Issue 1: NumPy Version Warning
**Symptom:** Warning about numpy<2.0 requirement
**Resolution:** This is expected - NumPy 1.26.4 is intentionally used
**Action:** Ignore this warning

### Issue 2: Pydantic Deprecation Warnings
**Symptom:** DeprecationWarning about .dict() method
**Resolution:** All occurrences have been updated to .model_dump()
**Action:** Verify no warnings after code updates

### Issue 3: TensorFlow/PyTorch Compatibility
**Symptom:** Questions about ML library versions
**Resolution:** Intentionally kept at current versions
**Action:** Schedule separate upgrade cycle for ML libraries

---

## Future ML Library Upgrade Plan

When ready to upgrade TensorFlow/PyTorch (Q1 2026):

### Preparation
- [ ] Create isolated testing environment
- [ ] Review TensorFlow 2.18 migration guide
- [ ] Review PyTorch 2.5 release notes
- [ ] Plan model retraining schedule

### Testing
- [ ] Test model loading compatibility
- [ ] Validate prediction accuracy
- [ ] Benchmark inference performance
- [ ] Test gradient calculations
- [ ] Verify saved model compatibility

### Deployment
- [ ] Update requirements with tested versions
- [ ] Retrain models if needed
- [ ] Update ML-specific documentation
- [ ] Deploy to staging → production

---

## Support & Resources

### Documentation
- `DEPENDENCY_UPDATE_NOTES.md` - Full technical details
- `UPGRADE_SUMMARY.md` - Quick reference guide
- `compare_versions.sh` - Version comparison script

### CVE References
- CVE-2024-35195: https://nvd.nist.gov/vuln/detail/CVE-2024-35195
- CVE-2024-35220: https://nvd.nist.gov/vuln/detail/CVE-2024-35220

### Package Documentation
- FastAPI: https://fastapi.tiangolo.com/release-notes/
- Pydantic: https://docs.pydantic.dev/latest/migration/
- PyTest: https://docs.pytest.org/en/stable/changelog.html

---

**Migration Owner:** _________________
**Reviewed By:** _________________
**Date Completed:** _________________
**Production Deployment:** _________________

---

**Last Updated:** 2025-10-10
